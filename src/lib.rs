use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;

pub mod video; // 内部实现模块 (frame/performance/extractor/utils) — 已由 core 重命名为 video
pub use video::{PyPerformanceResult, PyVideoFrame, VideoKeyframeExtractor};

// ---------------- Convenience PyFunctions (将注册到子模块 video) ----------------
#[pyfunction]
#[pyo3(signature = (video_path, output_dir, max_keyframes, max_save=None, ffmpeg_path=None, use_simd=None, threads=None, verbose=None, block_size=None))]
fn extract_keyframes_from_video(
    video_path: &str,
    output_dir: &str,
    max_keyframes: usize,
    max_save: Option<usize>,
    ffmpeg_path: Option<String>,
    use_simd: Option<bool>,
    threads: Option<usize>,
    verbose: Option<bool>,
    block_size: Option<usize>
) -> PyResult<PyPerformanceResult> {
    let extractor = VideoKeyframeExtractor::new(
        ffmpeg_path.unwrap_or_else(|| "ffmpeg".to_string()),
        threads.unwrap_or(0),
        verbose.unwrap_or(false)
    )?;
    extractor.process_video(
        video_path,
        output_dir,
        max_keyframes,
        max_save,      // max_save
        use_simd,      // use_simd
        block_size     // block_size
    )
}

#[pyfunction]
fn get_system_info(py: Python<'_>) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new_bound(py);
    dict.set_item("threads", rayon::current_num_threads())?;
    #[cfg(target_arch = "x86_64")]
    {
        dict.set_item("avx2_supported", std::arch::is_x86_feature_detected!("avx2"))?;
        dict.set_item("sse2_supported", std::arch::is_x86_feature_detected!("sse2"))?;
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        dict.set_item("simd_supported", false)?;
    }
    dict.set_item("version", env!("CARGO_PKG_VERSION"))?;
    Ok(dict.into())
}

// ---------------- 顶层模块 inkfox (导出子模块 video) ----------------
#[pymodule]
fn inkfox(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 创建子模块 video
    let video_mod = PyModule::new_bound(py, "video")?;
    video_mod.add_class::<PyVideoFrame>()?;
    video_mod.add_class::<PyPerformanceResult>()?;
    video_mod.add_class::<VideoKeyframeExtractor>()?;
    video_mod.add_function(wrap_pyfunction!(extract_keyframes_from_video, video_mod.clone())?)?;
    video_mod.add_function(wrap_pyfunction!(get_system_info, video_mod.clone())?)?;

    // 将子模块挂载到顶层 inkfox，并显式设置属性（部分环境 add_submodule 后属性缺失时冗余保障）
    m.add_submodule(&video_mod)?; // 等价于 m.add("video", &video_mod)
    if m.getattr("video").is_err() {
        m.setattr("video", &video_mod)?; // 保险：确保 inkfox.video 可访问
    }

    // 同时注册到 sys.modules，保证 "import inkfox.video" 以及工具链解析正常
    if let Ok(sys) = PyModule::import_bound(py, "sys") {
        if let Ok(modules_any) = sys.getattr("modules") {
            if let Ok(modules_dict) = modules_any.downcast::<PyDict>() {
                // 忽略设置失败（例如只读）——不影响核心功能
                let _ = modules_dict.set_item("inkfox.video", &video_mod);
            }
        }
    }

    // 直接在顶层再导出关键类型与函数，防止某些打包场景下无法访问子模块属性
    m.add_class::<PyVideoFrame>()?;
    m.add_class::<PyPerformanceResult>()?;
    m.add_class::<VideoKeyframeExtractor>()?;
    m.add_function(wrap_pyfunction!(extract_keyframes_from_video, m.clone())?)?;
    m.add_function(wrap_pyfunction!(get_system_info, m.clone())?)?;

    // 提供一个 helper 使得 inkfox.video 如果加载失败可以重新生成
    #[pyfn(m)]
    fn ensure_video_submodule(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
        if parent.getattr("video").is_err() {
            let vm = PyModule::new_bound(py, "video")?;
            vm.add_class::<PyVideoFrame>()?;
            vm.add_class::<PyPerformanceResult>()?;
            vm.add_class::<VideoKeyframeExtractor>()?;
            vm.add_function(wrap_pyfunction!(extract_keyframes_from_video, vm.clone())?)?;
            vm.add_function(wrap_pyfunction!(get_system_info, vm.clone())?)?;
            parent.add_submodule(&vm)?;
        }
        Ok(())
    }

    Ok(())
}
