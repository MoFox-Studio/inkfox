"""
Rust Video Keyframe Extractor - Python Type Hints

Ultra-fast video keyframe extraction tool with SIMD optimization.
"""

from typing import Dict, List, Optional, Tuple, Union, Any
from pathlib import Path

class PyVideoFrame:
    """
    Python绑定的视频帧结构
    
    表示一个视频帧，包含帧编号、尺寸和像素数据。
    """
    
    frame_number: int
    """帧编号"""
    
    width: int
    """帧宽度（像素）"""
    
    height: int
    """帧高度（像素）"""
    
    def __init__(self, frame_number: int, width: int, height: int, data: List[int]) -> None:
        """
        创建新的视频帧
        
        Args:
            frame_number: 帧编号
            width: 帧宽度
            height: 帧高度
            data: 像素数据（灰度值列表）
        """
        ...
    
    def get_data(self) -> List[int]:
        """
        获取帧的像素数据
        
        Returns:
            像素数据列表（灰度值）
        """
        ...
    
    def calculate_difference(self, other: 'PyVideoFrame') -> float:
        """
        计算与另一帧的差异
        
        Args:
            other: 要比较的另一帧
            
        Returns:
            帧差异值（0-255范围）
        """
        ...
    
    def calculate_difference_simd(self, other: 'PyVideoFrame', block_size: Optional[int] = None) -> float:
        """
        使用SIMD优化计算帧差异
        
        Args:
            other: 要比较的另一帧
            block_size: 处理块大小，默认8192
            
        Returns:
            帧差异值（0-255范围）
        """
        ...

class PyPerformanceResult:
    """
    性能测试结果
    
    包含详细的性能统计信息。
    """
    
    test_name: str
    """测试名称"""
    
    video_file: str
    """视频文件名"""
    
    total_time_ms: float
    """总处理时间（毫秒）"""
    
    frame_extraction_time_ms: float
    """帧提取时间（毫秒）"""
    
    keyframe_analysis_time_ms: float
    """关键帧分析时间（毫秒）"""
    
    total_frames: int
    """总帧数"""
    
    keyframes_extracted: int
    """提取的关键帧数"""
    
    keyframe_ratio: float
    """关键帧比例（百分比）"""
    
    processing_fps: float
    """处理速度（帧每秒）"""
    
    threshold: float
    """检测阈值"""
    
    optimization_type: str
    """优化类型"""
    
    simd_enabled: bool
    """是否启用SIMD"""
    
    threads_used: int
    """使用的线程数"""
    
    timestamp: str
    """时间戳"""
    
    def to_dict(self) -> Dict[str, Any]:
        """
        转换为Python字典
        
        Returns:
            包含所有结果字段的字典
        """
        ...

class VideoKeyframeExtractor:
    """
    主要的视频关键帧提取器类
    
    提供完整的视频关键帧提取功能，包括SIMD优化和多线程处理。
    """
    
    def __init__(
        self,
        ffmpeg_path: str = "ffmpeg",
        threads: int = 0,
        verbose: bool = False
    ) -> None:
        """
        创建关键帧提取器
        
        Args:
            ffmpeg_path: FFmpeg可执行文件路径，默认"ffmpeg"
            threads: 线程数，0表示自动检测
            verbose: 是否启用详细输出
        """
        ...
    
    def extract_frames(
        self,
        video_path: str,
        max_frames: Optional[int] = None
    ) -> Tuple[List[PyVideoFrame], int, int]:
        """
        从视频中提取帧
        
        Args:
            video_path: 视频文件路径
            max_frames: 最大提取帧数，None表示提取所有帧
            
        Returns:
            (帧列表, 宽度, 高度)
        """
        ...
    
    def extract_keyframes(
        self,
        frames: List[PyVideoFrame],
        threshold: float,
        use_simd: Optional[bool] = None,
        block_size: Optional[int] = None
    ) -> List[int]:
        """
        提取关键帧索引
        
        Args:
            frames: 视频帧列表
            threshold: 检测阈值
            use_simd: 是否使用SIMD优化，默认True
            block_size: 处理块大小，默认8192
            
        Returns:
            关键帧索引列表
        """
        ...
    
    def save_keyframes(
        self,
        video_path: str,
        keyframe_indices: List[int],
        output_dir: str,
        max_save: Optional[int] = None
    ) -> int:
        """
        保存关键帧为图片
        
        Args:
            video_path: 原视频文件路径
            keyframe_indices: 关键帧索引列表
            output_dir: 输出目录
            max_save: 最大保存数量，默认50
            
        Returns:
            实际保存的关键帧数量
        """
        ...
    
    def benchmark(
        self,
        video_path: str,
        threshold: float,
        test_name: str,
        max_frames: Optional[int] = None,
        use_simd: Optional[bool] = None,
        block_size: Optional[int] = None
    ) -> PyPerformanceResult:
        """
        运行性能测试
        
        Args:
            video_path: 视频文件路径
            threshold: 检测阈值
            test_name: 测试名称
            max_frames: 最大处理帧数，默认1000
            use_simd: 是否使用SIMD优化，默认True
            block_size: 处理块大小，默认8192
            
        Returns:
            性能测试结果
        """
        ...
    
    def process_video(
        self,
        video_path: str,
        output_dir: str,
        threshold: Optional[float] = None,
        max_frames: Optional[int] = None,
        max_save: Optional[int] = None,
        use_simd: Optional[bool] = None,
        block_size: Optional[int] = None
    ) -> PyPerformanceResult:
        """
        完整的处理流程
        
        执行完整的视频关键帧提取和保存流程。
        
        Args:
            video_path: 视频文件路径
            output_dir: 输出目录
            threshold: 检测阈值，默认2.0
            max_frames: 最大处理帧数，0表示处理所有帧
            max_save: 最大保存数量，默认50
            use_simd: 是否使用SIMD优化，默认True
            block_size: 处理块大小，默认8192
            
        Returns:
            处理结果
        """
        ...
    
    def get_cpu_features(self) -> Dict[str, bool]:
        """
        获取CPU特性信息
        
        Returns:
            CPU特性字典，包含AVX2、SSE2等支持信息
        """
        ...
    
    def get_thread_count(self) -> int:
        """
        获取当前配置的线程数
        
        Returns:
            配置的线程数
        """
        ...
    
    def get_configured_threads(self) -> int:
        """
        获取配置的线程数
        
        Returns:
            配置的线程数
        """
        ...
    
    def get_actual_thread_count(self) -> int:
        """
        获取实际运行的线程数
        
        Returns:
            实际运行的线程数
        """
        ...

def extract_keyframes_from_video(
    video_path: str,
    output_dir: str,
    threshold: Optional[float] = None,
    max_frames: Optional[int] = None,
    max_save: Optional[int] = None,
    ffmpeg_path: Optional[str] = None,
    use_simd: Optional[bool] = None,
    threads: Optional[int] = None,
    verbose: Optional[bool] = None
) -> PyPerformanceResult:
    """
    便捷函数：从视频提取关键帧
    
    这是一个便捷函数，封装了完整的关键帧提取流程。
    
    Args:
        video_path: 视频文件路径
        output_dir: 输出目录
        threshold: 检测阈值，默认2.0
        max_frames: 最大处理帧数，0表示处理所有帧
        max_save: 最大保存数量，默认50
        ffmpeg_path: FFmpeg路径，默认"ffmpeg"
        use_simd: 是否使用SIMD优化，默认True
        threads: 线程数，0表示自动检测
        verbose: 是否启用详细输出，默认False
        
    Returns:
        处理结果
        
    Example:
        >>> result = extract_keyframes_from_video(
        ...     "video.mp4",
        ...     "./output",
        ...     threshold=2.5,
        ...     max_save=30,
        ...     verbose=True
        ... )
        >>> print(f"提取了 {result.keyframes_extracted} 个关键帧")
    """
    ...

def get_system_info() -> Dict[str, Any]:
    """
    获取系统信息
    
    Returns:
        系统信息字典，包含：
        - threads: 可用线程数
        - avx2_supported: 是否支持AVX2（x86_64）
        - sse2_supported: 是否支持SSE2（x86_64）
        - simd_supported: 是否支持SIMD（非x86_64）
        - version: 库版本
        
    Example:
        >>> info = get_system_info()
        >>> print(f"线程数: {info['threads']}")
        >>> print(f"AVX2支持: {info.get('avx2_supported', False)}")
    """
    ...

# 类型别名
VideoPath = Union[str, Path]
"""视频文件路径类型"""

OutputPath = Union[str, Path]
"""输出路径类型"""

FrameData = List[int]
"""帧数据类型（像素值列表）"""

KeyframeIndices = List[int]
"""关键帧索引类型"""

# 常量
DEFAULT_THRESHOLD: float = 2.0
"""默认检测阈值"""

DEFAULT_BLOCK_SIZE: int = 8192
"""默认处理块大小"""

DEFAULT_MAX_SAVE: int = 50
"""默认最大保存数量"""

MAX_FRAME_DIFFERENCE: float = 255.0
"""最大帧差异值"""

# 版本信息
__version__: str = "0.1.0"
"""库版本"""
