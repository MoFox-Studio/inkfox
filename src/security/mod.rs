//! 安全模块：提供 API 密钥生成和验证功能
//!
//! 此模块实现了基于加密安全随机数的 API 密钥生成，
//! 以及基于 HMAC-SHA256 的密钥验证机制。

use pyo3::prelude::*;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// 生成加密安全的 API 密钥
///
/// # Arguments
///
/// * `length` - 密钥字节长度（建议至少 32 字节）
///
/// # Returns
///
/// 返回 Base64 编码的密钥字符串
#[pyfunction]
#[pyo3(signature = (length=32))]
pub fn generate_api_key(length: usize) -> PyResult<String> {
    if length < 16 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "密钥长度至少为 16 字节",
        ));
    }

    let mut rng = rand::thread_rng();
    let mut key_bytes = vec![0u8; length];
    rng.fill_bytes(&mut key_bytes);

    // 使用 Base64 编码，确保可打印
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &key_bytes,
    ))
}

/// 生成带时间戳的 API 密钥
///
/// 密钥格式：timestamp_base64(random_bytes)
/// 这样可以在日志中识别密钥的生成时间
#[pyfunction]
#[pyo3(signature = (length=32))]
pub fn generate_timestamped_api_key(length: usize) -> PyResult<String> {
    if length < 16 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "密钥长度至少为 16 字节",
        ));
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("时间错误: {}", e)))?
        .as_secs();

    let mut rng = rand::thread_rng();
    let mut key_bytes = vec![0u8; length];
    rng.fill_bytes(&mut key_bytes);

    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &key_bytes,
    );

    Ok(format!("{}_{}", timestamp, encoded))
}

/// 验证 API 密钥
///
/// # Arguments
///
/// * `api_key` - 待验证的密钥
/// * `valid_keys` - 有效密钥列表
///
/// # Returns
///
/// 如果密钥有效返回 true，否则返回 false
#[pyfunction]
pub fn verify_api_key(api_key: &str, valid_keys: Vec<String>) -> PyResult<bool> {
    if api_key.is_empty() {
        return Ok(false);
    }

    // 使用常量时间比较，防止时序攻击
    for valid_key in valid_keys.iter() {
        if constant_time_compare(api_key.as_bytes(), valid_key.as_bytes()) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// 常量时间字符串比较，防止时序攻击
///
/// # Arguments
///
/// * `a` - 第一个字节数组
/// * `b` - 第二个字节数组
///
/// # Returns
///
/// 如果两个数组完全相同返回 true，否则返回 false
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// 生成 API 密钥哈希（用于存储）
///
/// # Arguments
///
/// * `api_key` - 原始密钥
///
/// # Returns
///
/// 返回密钥的 SHA256 哈希值（十六进制字符串）
#[pyfunction]
pub fn hash_api_key(api_key: &str) -> PyResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// 验证 API 密钥哈希
///
/// # Arguments
///
/// * `api_key` - 待验证的密钥
/// * `key_hash` - 存储的密钥哈希
///
/// # Returns
///
/// 如果密钥匹配返回 true，否则返回 false
#[pyfunction]
pub fn verify_api_key_hash(api_key: &str, key_hash: &str) -> PyResult<bool> {
    let computed_hash = hash_api_key(api_key)?;
    Ok(constant_time_compare(
        computed_hash.as_bytes(),
        key_hash.as_bytes(),
    ))
}

/// Python 模块绑定
pub fn register_security_module(py: Python<'_>, parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let security_mod = PyModule::new_bound(py, "security")?;
    
    security_mod.add_function(wrap_pyfunction!(generate_api_key, &security_mod)?)?;
    security_mod.add_function(wrap_pyfunction!(generate_timestamped_api_key, &security_mod)?)?;
    security_mod.add_function(wrap_pyfunction!(verify_api_key, &security_mod)?)?;
    security_mod.add_function(wrap_pyfunction!(hash_api_key, &security_mod)?)?;
    security_mod.add_function(wrap_pyfunction!(verify_api_key_hash, &security_mod)?)?;

    parent_module.add_submodule(&security_mod)?;
    
    Ok(())
}
