//! inkfox: PyO3 绑定。遵循官方文档简洁模式：使用 `add_submodule` 让解释器注册子模块。
//! 不再显式写 `sys.modules`，保持最小实现，便于排查问题。

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;

pub mod memory;
pub use memory::PyMetadataIndex;

pub mod video; // 视频相关
pub use video::{PyPerformanceResult, PyVideoFrame, VideoKeyframeExtractor};

// -------------------------------------------------------------------------------------------------
// 辅助：创建并注册子模块
// -------------------------------------------------------------------------------------------------
// 官方模式：直接 new 子模块 -> add_submodule；解释器会完成合适注册。

// -------------------------------------------------------------------------------------------------
// 辅助函数: 便捷调用
// -------------------------------------------------------------------------------------------------
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
    block_size: Option<usize>,
) -> PyResult<PyPerformanceResult> {
    let extractor = VideoKeyframeExtractor::new(
        ffmpeg_path.unwrap_or_else(|| "ffmpeg".to_string()),
        threads.unwrap_or(0),
        verbose.unwrap_or(false),
    )?;
    extractor.process_video(
        video_path,
        output_dir,
        max_keyframes,
        max_save,
        use_simd,
        block_size,
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

// -------------------------------------------------------------------------------------------------
// 顶层模块
// -------------------------------------------------------------------------------------------------
#[pymodule]
fn inkfox(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 顶层直接暴露的类型与函数（兼容旧使用方式）
    m.add_class::<PyVideoFrame>()?;
    m.add_class::<PyPerformanceResult>()?;
    m.add_class::<VideoKeyframeExtractor>()?;
    m.add_class::<PyMetadataIndex>()?;
    m.add_function(wrap_pyfunction!(extract_keyframes_from_video, m.clone())?)?;
    m.add_function(wrap_pyfunction!(get_system_info, m.clone())?)?;

    // 子模块: video
    let video_mod = PyModule::new_bound(py, "video")?;
    video_mod.add_class::<PyVideoFrame>()?;
    video_mod.add_class::<PyPerformanceResult>()?;
    video_mod.add_class::<VideoKeyframeExtractor>()?;
    video_mod.add_function(wrap_pyfunction!(extract_keyframes_from_video, m.clone())?)?;
    video_mod.add_function(wrap_pyfunction!(get_system_info, m.clone())?)?;
    let video_all = PyList::new_bound(py, [
        "PyVideoFrame",
        "PyPerformanceResult",
        "VideoKeyframeExtractor",
        "extract_keyframes_from_video",
        "get_system_info",
    ]);
    video_mod.setattr("__all__", video_all)?;
    m.add_submodule(&video_mod)?;

    // 子模块: memory
    let memory_mod = PyModule::new_bound(py, "memory")?;
    memory_mod.add_class::<PyMetadataIndex>()?;
    let memory_all = PyList::new_bound(py, ["PyMetadataIndex"]);
    memory_mod.setattr("__all__", memory_all)?;
    m.add_submodule(&memory_mod)?;

    // 设置 __path__ 为 None，表示命名空间包
    m.setattr("__path__", py.None())?;

    // 修正类型 __module__ 到各自子模块
    py.get_type_bound::<PyVideoFrame>().setattr("__module__", "inkfox.video").ok();
    py.get_type_bound::<PyPerformanceResult>().setattr("__module__", "inkfox.video").ok();
    py.get_type_bound::<VideoKeyframeExtractor>().setattr("__module__", "inkfox.video").ok();
    py.get_type_bound::<PyMetadataIndex>().setattr("__module__", "inkfox.memory").ok();

    // 顶层 __all__
    let top_all = PyList::new_bound(py, [
        "PyMetadataIndex",
        "PyVideoFrame",
        "PyPerformanceResult",
        "VideoKeyframeExtractor",
        "extract_keyframes_from_video",
        "get_system_info",
    ]);
    m.setattr("__all__", top_all)?;

    Ok(())
}
