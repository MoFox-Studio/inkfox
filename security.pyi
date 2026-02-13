"""inkfox.security submodule type stubs"""
from __future__ import annotations
from typing import List

def generate_api_key(length: int = 32) -> str:
    """
    生成加密安全的 API 密钥
    
    Args:
        length: 密钥字节长度（建议至少 32 字节，最小 16 字节）
    
    Returns:
        Base64 编码的密钥字符串
    
    Raises:
        ValueError: 如果 length < 16
    """
    ...

def generate_timestamped_api_key(length: int = 32) -> str:
    """
    生成带时间戳的 API 密钥
    
    密钥格式：timestamp_base64(random_bytes)
    
    Args:
        length: 密钥字节长度（建议至少 32 字节，最小 16 字节）
    
    Returns:
        带时间戳前缀的密钥字符串
    
    Raises:
        ValueError: 如果 length < 16
    """
    ...

def verify_api_key(api_key: str, valid_keys: List[str]) -> bool:
    """
    验证 API 密钥（使用常量时间比较，防止时序攻击）
    
    Args:
        api_key: 待验证的密钥
        valid_keys: 有效密钥列表
    
    Returns:
        如果密钥有效返回 True，否则返回 False
    """
    ...

def hash_api_key(api_key: str) -> str:
    """
    生成 API 密钥哈希（SHA-256）
    
    Args:
        api_key: 原始密钥
    
    Returns:
        密钥的 SHA256 哈希值（十六进制字符串）
    """
    ...

def verify_api_key_hash(api_key: str, key_hash: str) -> bool:
    """
    验证 API 密钥哈希（使用常量时间比较）
    
    Args:
        api_key: 待验证的密钥
        key_hash: 存储的密钥哈希
    
    Returns:
        如果密钥匹配返回 True，否则返回 False
    """
    ...

__all__ = [
    "generate_api_key",
    "generate_timestamped_api_key",
    "verify_api_key",
    "hash_api_key",
    "verify_api_key_hash",
]
