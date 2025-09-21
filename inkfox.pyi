"""Type hints for the native extension module `inkfox`.

The compiled module dynamically creates a submodule attribute `video` at runtime.
Because it is not a package, static analyzers can't resolve `from inkfox import video`.
We therefore declare `video` here as a module attribute.
"""

from __future__ import annotations
from typing import Any, Dict
import types as _types

# Runtime-created submodule (Python attribute on the extension). Not a package path.
video: _types.ModuleType  # type: ignore

__all__ = [
    "video",
    "get_system_info",
    "extract_keyframes_from_video",
    "PyVideoFrame",
    "PyPerformanceResult",
    "VideoKeyframeExtractor",
]

def get_system_info() -> Dict[str, Any]:
    """Return system / build information (threads, SIMD flags, version)."""
    ...

def extract_keyframes_from_video(
    video_path: str,
    output_dir: str,
    max_keyframes: int,
    max_save: int | None = None,
    ffmpeg_path: str | None = None,
    use_simd: bool | None = None,
    threads: int | None = None,
    verbose: bool | None = None,
    block_size: int | None = None,
):
    """Convenience end‑to‑end keyframe extraction.

    Returns:
        PyPerformanceResult
    """
    ...

# Re-exports (mirrors video.*)
from .video import PyVideoFrame, PyPerformanceResult, VideoKeyframeExtractor  # type: ignore

__version__: str
