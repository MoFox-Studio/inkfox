use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use dashmap::DashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub memory_id: String,
    pub user_id: String,
    pub memory_type: String,
    pub subjects: Vec<String>,
    pub objects: Vec<String>,
    pub keywords: Vec<String>,
    pub tags: Vec<String>,
    pub importance: u8,
    pub confidence: u8,
    pub created_at: f64,
    pub access_count: u32,
    pub chat_id: Option<String>,
    pub content_preview: Option<String>,
}

impl MetadataEntry {
    fn normalize_vec(raw: &Bound<PyAny>) -> Vec<String> {
        if let Ok(list) = raw.downcast::<PyList>() {
            list.iter().filter_map(|v| v.extract::<String>().ok()).collect()
        } else if let Ok(s) = raw.extract::<String>() { 
            vec![s] 
        } else { 
            vec![] 
        }
    }

    fn from_pydict(d: &Bound<PyDict>) -> PyResult<Self> {
        fn get_item<'py>(d: &Bound<'py, PyDict>, key: &str) -> Option<Bound<'py, PyAny>> {
            match d.get_item(key) {
                Ok(Some(v)) => Some(v),
                _ => None,
            }
        }
        fn get_str(d: &Bound<PyDict>, key: &str) -> String {
            get_item(d, key).and_then(|v| v.extract::<String>().ok()).unwrap_or_default()
        }
        fn get_u8(d: &Bound<PyDict>, key: &str, default: u8) -> u8 {
            get_item(d, key).and_then(|v| v.extract::<u8>().ok()).unwrap_or(default)
        }
        fn get_u32(d: &Bound<PyDict>, key: &str, default: u32) -> u32 {
            get_item(d, key).and_then(|v| v.extract::<u32>().ok()).unwrap_or(default)
        }
        fn get_vec(d: &Bound<PyDict>, key: &str) -> Vec<String> {
            get_item(d, key).map(|v| MetadataEntry::normalize_vec(&v)).unwrap_or_default()
        }
        
        let created_at: f64 = get_item(d, "created_at")
            .and_then(|v| v.extract::<f64>().ok())
            .unwrap_or_else(|| SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64());
        
        let memory_id = get_str(d, "memory_id");
        if memory_id.is_empty() { 
            return Err(pyo3::exceptions::PyValueError::new_err("memory_id missing")); 
        }
        
        Ok(MetadataEntry {
            memory_id,
            user_id: get_str(d, "user_id"),
            memory_type: get_str(d, "memory_type"),
            subjects: get_vec(d, "subjects"),
            objects: get_vec(d, "objects"),
            keywords: get_vec(d, "keywords"),
            tags: get_vec(d, "tags"),
            importance: get_u8(d, "importance", 2),
            confidence: get_u8(d, "confidence", 2),
            created_at,
            access_count: get_u32(d, "access_count", 0),
            chat_id: get_item(d, "chat_id").and_then(|v| v.extract::<String>().ok()),
            content_preview: get_item(d, "content_preview").and_then(|v| v.extract::<String>().ok()),
        })
    }
}

#[derive(Default)]
struct InnerIndex {
    entries: DashMap<String, MetadataEntry>,
    type_index: DashMap<String, Vec<String>>,
    subject_index: DashMap<String, Vec<String>>,
    keyword_index: DashMap<String, Vec<String>>,
    tag_index: DashMap<String, Vec<String>>,
}

impl InnerIndex {
    fn add_entry(&self, e: MetadataEntry) {
        let id = e.memory_id.clone();
        
        // Remove old references if exists
        if let Some(old) = self.entries.get(&id) {
            self.remove_from_inverted(old.value());
        }
        
        self.entries.insert(id.clone(), e.clone());
        
        // Add to inverted indices
        self.update_index(&self.type_index, &e.memory_type, &id);
        
        for s in &e.subjects { 
            self.update_index(&self.subject_index, &s.to_lowercase(), &id); 
        }
        for k in &e.keywords { 
            self.update_index(&self.keyword_index, &k.to_lowercase(), &id); 
        }
        for t in &e.tags { 
            self.update_index(&self.tag_index, &t.to_lowercase(), &id); 
        }
    }
    
    fn update_index(&self, map: &DashMap<String, Vec<String>>, key: &str, id: &str) {
        if key.is_empty() { return; }
        
        map.entry(key.to_string())
            .and_modify(|v| {
                if !v.iter().any(|x| x == id) {
                    v.push(id.to_string());
                }
            })
            .or_insert_with(|| vec![id.to_string()]);
    }
    
    fn remove_from_inverted(&self, e: &MetadataEntry) {
        self.remove_from_index(&self.type_index, &e.memory_type, &e.memory_id);
        
        for s in &e.subjects { 
            self.remove_from_index(&self.subject_index, &s.to_lowercase(), &e.memory_id); 
        }
        for k in &e.keywords { 
            self.remove_from_index(&self.keyword_index, &k.to_lowercase(), &e.memory_id); 
        }
        for t in &e.tags { 
            self.remove_from_index(&self.tag_index, &t.to_lowercase(), &e.memory_id); 
        }
    }
    
    fn remove_from_index(&self, map: &DashMap<String, Vec<String>>, key: &str, id: &str) {
        if let Some(mut entry) = map.get_mut(key) {
            entry.retain(|x| x != id);
            // Remove empty vectors to save memory
            if entry.is_empty() {
                map.remove(key);
            }
        }
    }
}

#[pyclass]
pub struct PyMetadataIndex {
    inner: Arc<InnerIndex>,
    path: Option<PathBuf>,
}

#[pymethods]
impl PyMetadataIndex {
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn new(path: Option<String>) -> PyResult<Self> {
        let idx = InnerIndex::default();
        let inst = PyMetadataIndex { 
            inner: Arc::new(idx), 
            path: path.map(PathBuf::from) 
        };
        
        if let Some(p) = &inst.path {
            if p.exists() {
                if let Ok(bytes) = fs::read(p) {
                    if let Ok(vec) = serde_json::from_slice::<Vec<MetadataEntry>>(&bytes) {
                        for e in vec { 
                            inst.inner.add_entry(e); 
                        }
                    }
                }
            }
        }
        Ok(inst)
    }

    pub fn batch_add(&self, _py: Python<'_>, entries: &Bound<PyAny>) -> PyResult<usize> {
        let mut count = 0usize;
        if let Ok(list) = entries.downcast::<PyList>() {
            for item in list.iter() {
                if let Ok(d) = item.downcast::<PyDict>() { 
                    if let Ok(e) = MetadataEntry::from_pydict(&d) { 
                        self.inner.add_entry(e); 
                        count += 1; 
                    } 
                }
            }
        }
        Ok(count)
    }

    #[pyo3(signature = (params))]
    pub fn search_flexible(&self, params: &Bound<PyDict>) -> PyResult<Vec<String>> {
        let get = |k: &str| params.get_item(k).ok().flatten();
        let user_id: Option<String> = get("user_id").and_then(|v| v.extract::<String>().ok());
        let types: Option<Vec<String>> = get("memory_types").and_then(|v| v.extract::<Vec<String>>().ok());
        let subjects: Option<Vec<String>> = get("subjects").and_then(|v| v.extract::<Vec<String>>().ok());
        let created_after: Option<f64> = get("created_after").and_then(|v| v.extract::<f64>().ok());
        let created_before: Option<f64> = get("created_before").and_then(|v| v.extract::<f64>().ok());
        let limit: usize = get("limit").and_then(|v| v.extract::<usize>().ok()).unwrap_or(100);

        let subjects_lc: Vec<String> = subjects.clone()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();

        let mut scored: Vec<(f32, String, f64)> = self.inner.entries
            .par_iter()
            .filter_map(|entry| {
                let e = entry.value();
                
                // User filtering
                if let Some(uid) = &user_id { 
                    if &e.user_id != uid { 
                        return None; 
                    } 
                }
                
                // Time filtering with proper float comparison
                if let Some(ca) = created_after { 
                    if e.created_at.total_cmp(&ca).is_lt() { 
                        return None; 
                    } 
                }
                if let Some(cb) = created_before { 
                    if e.created_at.total_cmp(&cb).is_gt() { 
                        return None; 
                    } 
                }

                let mut score = 0f32;
                
                // Type matching
                if let Some(ts) = &types { 
                    if !ts.is_empty() { 
                        let mut tscore = 0f32; 
                        for t in ts { 
                            if e.memory_type == *t { 
                                tscore = 1.0; 
                                break; 
                            } else if e.memory_type.contains(t) || t.contains(&e.memory_type) { 
                                tscore = 0.5; 
                                break; 
                            } 
                        } 
                        score += tscore; 
                    } 
                }
                
                // Subject matching
                if !subjects_lc.is_empty() { 
                    let mut sscore = 0f32; 
                    for s in &subjects_lc { 
                        for es in &e.subjects { 
                            let esn = es.to_lowercase(); 
                            if esn == *s { 
                                sscore = 1.0; 
                                break; 
                            } 
                            if esn.contains(s) || s.contains(&esn) { 
                                sscore = 0.6; 
                                break; 
                            } 
                        } 
                        if sscore > 0.0 { 
                            break; 
                        } 
                    } 
                    score += sscore; 
                }
                
                // Object-subject relation
                if !subjects_lc.is_empty() && !e.objects.is_empty() { 
                    let mut os = 0f32; 
                    'outer: for o in &e.objects { 
                        let on = o.to_lowercase(); 
                        for s in &subjects_lc { 
                            if on.contains(s) || s.contains(&on) { 
                                os = 0.8; 
                                break 'outer; 
                            } 
                        } 
                    } 
                    score += os; 
                }
                
                // Time bonus
                if created_after.is_some() || created_before.is_some() { 
                    score += 1.0; 
                }
                
                if score >= 2.0 { 
                    Some((score, e.memory_id.clone(), e.created_at)) 
                } else { 
                    None 
                }
            })
            .collect();

        // Sort by score (desc) then time (desc)
        scored.par_sort_unstable_by(|a, b| {
            b.0.total_cmp(&a.0)
                .then_with(|| b.2.total_cmp(&a.2))
        });
        
        Ok(scored.into_iter().take(limit).map(|(_, id, _)| id).collect())
    }

    #[pyo3(signature = (params))]
    pub fn search_strict(&self, params: &Bound<PyDict>) -> PyResult<Vec<String>> {
        let get = |k: &str| params.get_item(k).ok().flatten();
        let user_id: Option<String> = get("user_id").and_then(|v| v.extract::<String>().ok());
        let types: Option<Vec<String>> = get("memory_types").and_then(|v| v.extract::<Vec<String>>().ok());
        let subjects: Option<Vec<String>> = get("subjects").and_then(|v| v.extract::<Vec<String>>().ok());
        let keywords: Option<Vec<String>> = get("keywords").and_then(|v| v.extract::<Vec<String>>().ok());
        let tags: Option<Vec<String>> = get("tags").and_then(|v| v.extract::<Vec<String>>().ok());
        let importance_min: Option<u8> = get("importance_min").and_thenv| v.extract::<u8>().ok());
        let importance_max: Option<u8> = get("importance_max").and_then(|v| v.extract::<u8>().ok());
        let created_after: Option<f64> = get("created_after").and_then(|v| v.extract::<f64>().ok());
        let created_before: Option<f64> = get("created_before").and_then(|v| v.extract::<f64>().ok());
        let limit: usize = get("limit").and_then(|v| v.extract::<usize>().ok()).unwrap_or(100);

        let mut candidates: Vec<(String, f64)> = self.inner.entries
            .iter()
            .filter_map(|entry| {
                let e = entry.value();
                
                // User filtering
                if let Some(uid) = &user_id { 
                    if &e.user_id != uid { 
                        return None; 
                    } 
                }
                
                // Type filtering
                if let Some(ts) = &types { 
                    if !ts.is_empty() && !ts.iter().any(|t| t == &e.memory_type) { 
                        return None; 
                    } 
                }
                
                // Subject filtering (fuzzy)
                if let Some(subs) = &subjects { 
                    if !subs.is_empty() { 
                        let mut found = false;
                        for s in subs { 
                            let s_lc = s.to_lowercase();
                            for es in &e.subjects { 
                                let es_lc = es.to_lowercase();
                                if es_lc == s_lc || es_lc.contains(&s_lc) || s_lc.contains(&es_lc) { 
                                    found = true;
                                    break;
                                } 
                            }
                            if found { break; }
                        }
                        if !found { return None; }
                    } 
                }
                
                // Keyword filtering
                if let Some(ks) = &keywords { 
                    if !ks.is_empty() { 
                        let mut found = false;
                        for k in ks { 
                            let k_lc = k.to_lowercase();
                            for ek in &e.keywords { 
                                let ek_lc = ek.to_lowercase();
                                if ek_lc == k_lc || ek_lc.contains(&k_lc) || k_lc.contains(&ek_lc) { 
                                    found = true;
                                    break;
                                } 
                            }
                            if found { break; }
                        }
                        if !found { return None; }
                    } 
                }
                
                // Tag filtering (exact OR)
                if let Some(ts) = &tags { 
                    if !ts.is_empty() && !e.tags.iter().any(|t| ts.iter().any(|tt| tt == t)) { 
                        return None; 
                    } 
                }
                
                // Importance filtering
                if let Some(mi) = importance_min { 
                    if e.importance < mi { 
                        return None; 
                    } 
                }
                if let Some(ma) = importance_max { 
                    if e.importance > ma { 
                        return None; 
                    } 
                }
                
                // Time range filtering
                if let Some(ca) = created_after { 
                    if e.created_at.total_cmp(&ca).is_lt() { 
                        return None; 
                    } 
                }
                if let Some(cb) = created_before { 
                    if e.created_at.total_cmp(&cb).is_gt() { 
                        return None; 
                    } 
                }
                
                Some((e.memory_id.clone(), e.created_at))
            })
            .collect();

        // Sort by time descending
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        
        Ok(candidates.into_iter().take(limit).map(|(id, _)| id).collect())
    }

    pub fn stats(&self) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let d = PyDict::new_bound(py);
            d.set_item("total", self.inner.entries.len())?;
            d.set_item("types_indexed", self.inner.type_index.len())?;
            d.set_item("subjects_indexed", self.inner.subject_index.len())?;
            d.set_item("keywords_indexed", self.inner.keyword_index.len())?;
            d.set_item("tags_indexed", self.inner.tag_index.len())?;
            
            let types_dist = PyDict::new_bound(py);
            for kv in self.inner.type_index.iter() {
                types_dist.set_item(kv.key(), kv.value().len())?;
            }
            d.set_item("types_dist", types_dist)?;
            
            Ok(d.into())
        })
    }

    pub fn save(&self) -> PyResult<bool> {
        if let Some(p) = &self.path { 
            let list: Vec<MetadataEntry> = self.inner.entries.iter().map(|e| e.value().clone()).collect();
            if let Ok(json) = serde_json::to_vec(&list) { 
                if let Err(_) = fs::write(p, json) {
                    return Ok(false);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }
}
