"""Tests for Python plugins."""

import pytest
from plugins.handshake_logger import HandshakeLogger
from plugins.stats_display import StatsDisplay


@pytest.mark.asyncio
async def test_handshake_logger_basic() -> None:
    """Test basic handshake logger functionality."""
    plugin = HandshakeLogger()
    
    assert plugin.name() == "handshake_logger"
    assert plugin.version() == "1.0.0"
    assert plugin.handshake_count == 0


@pytest.mark.asyncio
async def test_handshake_logger_on_handshake() -> None:
    """Test handshake logging."""
    plugin = HandshakeLogger()
    
    ap = {
        "bssid": "aa:bb:cc:dd:ee:ff",
        "essid": "TestNetwork",
        "channel": 6,
        "rssi": -45,
    }
    
    station = {
        "mac": "11:22:33:44:55:66",
        "rssi": -60,
    }
    
    await plugin.on_handshake("/tmp/test.pcap", ap, station)
    
    assert plugin.handshake_count == 1


@pytest.mark.asyncio
async def test_stats_display_basic() -> None:
    """Test basic stats display functionality."""
    plugin = StatsDisplay()
    
    assert plugin.name() == "stats_display"
    assert plugin.version() == "1.0.0"
    assert plugin.total_aps_seen == 0


@pytest.mark.asyncio
async def test_stats_display_wifi_update() -> None:
    """Test WiFi statistics tracking."""
    plugin = StatsDisplay()
    
    aps = [
        {"bssid": "aa:bb:cc:dd:ee:ff", "essid": "Network1", "channel": 6, "rssi": -45},
        {"bssid": "11:22:33:44:55:66", "essid": "Network2", "channel": 11, "rssi": -60},
        {"bssid": "ff:ee:dd:cc:bb:aa", "essid": "Network3", "channel": 6, "rssi": -70},
    ]
    
    await plugin.on_wifi_update(aps)
    
    assert plugin.total_aps_seen == 3
    assert 6 in plugin.channels_used
    assert 11 in plugin.channels_used
    assert plugin.strongest_ap_rssi == -45
    assert plugin.strongest_ap_essid == "Network1"


@pytest.mark.asyncio
async def test_plugin_lifecycle() -> None:
    """Test plugin lifecycle hooks."""
    plugin = HandshakeLogger()
    
    # Test lifecycle methods don't crash
    await plugin.on_loaded()
    await plugin.on_ready()
    await plugin.on_starting()
    await plugin.on_unload()


@pytest.mark.asyncio
async def test_mood_change() -> None:
    """Test mood change handling."""
    plugin = HandshakeLogger()
    
    # Should not crash
    await plugin.on_mood_change("Ready", "Excited")


@pytest.mark.asyncio
async def test_epoch_handling() -> None:
    """Test epoch handling."""
    plugin = HandshakeLogger()
    
    epoch_data = {
        "epoch": 10,
        "handshakes": 2,
        "associations": 5,
        "deauths": 3,
    }
    
    # Should not crash
    await plugin.on_epoch(10, epoch_data)
