"""Example plugin that displays statistics."""

from typing import Any

from pwnagotchi_plugin import Plugin


class StatsDisplay(Plugin):
    """Plugin that tracks and displays statistics."""

    def __init__(self) -> None:
        """Initialize plugin."""
        self.total_aps_seen = 0
        self.channels_used: set[int] = set()
        self.strongest_ap_rssi = -100
        self.strongest_ap_essid = ""

    def name(self) -> str:
        """Return plugin name."""
        return "stats_display"

    def version(self) -> str:
        """Return plugin version."""
        return "1.0.0"

    def description(self) -> str:
        """Return plugin description."""
        return "Displays WiFi statistics"

    async def on_wifi_update(self, access_points: list[dict[str, Any]]) -> None:
        """
        Track WiFi statistics.

        Args:
            access_points: List of visible APs
        """
        self.total_aps_seen = len(access_points)

        for ap in access_points:
            channel = ap.get("channel", 0)
            self.channels_used.add(channel)

            rssi = ap.get("rssi", -100)
            if rssi > self.strongest_ap_rssi:
                self.strongest_ap_rssi = rssi
                self.strongest_ap_essid = ap.get("essid", "Unknown")

    async def on_epoch(self, epoch: int, epoch_data: dict[str, Any]) -> None:
        """
        Display statistics every 10 epochs.

        Args:
            epoch: Current epoch
            epoch_data: Epoch data
        """
        if epoch % 10 == 0:
            pass

    async def on_channel_hop(self, channel: int) -> None:
        """
        Log channel hops.

        Args:
            channel: New channel
        """
