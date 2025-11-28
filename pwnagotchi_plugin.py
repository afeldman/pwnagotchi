"""
Plugin base class and type definitions for Pwnagotchi Python plugins.

This module provides the base Plugin class and type definitions that
Python plugins should inherit from and use.
"""

from abc import ABC, abstractmethod
from typing import Any


class Plugin(ABC):
    """
    Base class for Pwnagotchi Python plugins.

    All plugins must inherit from this class and implement at minimum
    the name(), version(), and description() methods.

    Example:
        >>> class MyPlugin(Plugin):
        ...     def name(self) -> str:
        ...         return "my_plugin"
        ...
        ...     def version(self) -> str:
        ...         return "1.0.0"
        ...
        ...     def description(self) -> str:
        ...         return "Example plugin"
        ...
        ...     async def on_handshake(
        ...         self,
        ...         filename: str,
        ...         access_point: Dict[str, Any],
        ...         station: Dict[str, Any]
        ...     ) -> None:
        ...         print(f"Handshake: {filename}")
    """

    @abstractmethod
    def name(self) -> str:
        """
        Return the plugin name.

        Returns:
            Plugin name as string (no spaces, lowercase recommended)
        """
        ...

    @abstractmethod
    def version(self) -> str:
        """
        Return the plugin version.

        Returns:
            Version string (semantic versioning recommended: "1.0.0")
        """
        ...

    @abstractmethod
    def description(self) -> str:
        """
        Return the plugin description.

        Returns:
            Short description of what the plugin does
        """
        ...

    async def on_loaded(self) -> None:
        """Called when the plugin is loaded."""

    async def on_unload(self) -> None:
        """Called when the plugin is unloaded."""

    async def on_ready(self) -> None:
        """Called when the agent is ready."""

    async def on_starting(self) -> None:
        """Called when the agent is starting."""

    async def on_rebooting(self) -> None:
        """Called when the agent is rebooting."""

    async def on_handshake(
        self,
        filename: str,
        access_point: dict[str, Any],
        station: dict[str, Any],
    ) -> None:
        """
        Called when a handshake is captured.

        Args:
            filename: Path to the captured handshake file
            access_point: Dictionary with AP info (bssid, essid, channel, rssi)
            station: Dictionary with station info (mac, rssi)
        """

    async def on_epoch(self, epoch: int, epoch_data: dict[str, Any]) -> None:
        """
        Called on each epoch (main loop iteration).

        Args:
            epoch: Current epoch number
            epoch_data: Dictionary with epoch statistics
                - epoch: Current epoch number
                - handshakes: Handshakes captured this epoch
                - associations: Association attempts this epoch
                - deauths: Deauth attacks this epoch
        """

    async def on_mood_change(self, old_mood: str, new_mood: str) -> None:
        """
        Called when the agent's mood changes.

        Args:
            old_mood: Previous mood (Ready, Bored, Sad, Angry, Excited, etc.)
            new_mood: New mood
        """

    async def on_wifi_update(self, access_points: list[dict[str, Any]]) -> None:
        """
        Called when the WiFi list is updated.

        Args:
            access_points: List of AP dictionaries with keys:
                - bssid: AP MAC address
                - essid: Network name
                - channel: WiFi channel
                - rssi: Signal strength in dBm
        """

    async def on_wait(self, seconds: int) -> None:
        """
        Called when the agent is waiting.

        Args:
            seconds: Number of seconds to wait
        """

    async def on_sleep(self, seconds: int) -> None:
        """
        Called when the agent is sleeping.

        Args:
            seconds: Number of seconds to sleep
        """

    async def on_association(self, access_point: dict[str, Any]) -> None:
        """
        Called when sending an association frame.

        Args:
            access_point: Dictionary with AP info
        """

    async def on_deauthentication(
        self,
        access_point: dict[str, Any],
        station: dict[str, Any],
    ) -> None:
        """
        Called when deauthenticating a client.

        Args:
            access_point: Dictionary with AP info
            station: Dictionary with station info
        """

    async def on_channel_hop(self, channel: int) -> None:
        """
        Called when hopping to a new channel.

        Args:
            channel: New WiFi channel number
        """

    async def on_peer_detected(self, peer_fingerprint: str) -> None:
        """
        Called when a peer is detected.

        Args:
            peer_fingerprint: Unique fingerprint of the peer
        """

    async def on_peer_lost(self, peer_fingerprint: str) -> None:
        """
        Called when a peer is lost.

        Args:
            peer_fingerprint: Unique fingerprint of the peer
        """

    async def on_internet_available(self) -> None:
        """Called when internet connectivity is available."""

    async def on_bettercap_event(self, event_tag: str, event_data: Any) -> None:
        """
        Called on bettercap events.

        Args:
            event_tag: Event type tag
            event_data: Event payload data
        """
