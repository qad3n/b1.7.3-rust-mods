"""Copy the Wireless Redstone textures out of the user's own ModLoader zips into assets/wirelessredstone.

The six zips (WirelessRedstone, Clocker, PowerConfigurator, Remote, Sniffer, Triangulator) are passed as arguments or
as one folder holding them. The assets belong to their authors, so they are never committed. Run:
    python3 -I tools/extract_assets.py /path/to/Wireless\\ Redstone/
"""

import sys
import zipfile
from pathlib import Path

TARGET = Path(__file__).resolve().parent.parent / "assets" / "wirelessredstone"

# zip entry -> file under assets/wirelessredstone
FILES = [
    ("WirelessSprites/topOn.png", "textures/block/top_on.png"),
    ("WirelessSprites/topOff.png", "textures/block/top_off.png"),
    ("WirelessSprites/rxOn.png", "textures/block/receiver_on.png"),
    ("WirelessSprites/rxOff.png", "textures/block/receiver_off.png"),
    ("WirelessSprites/txOn.png", "textures/block/transmitter_on.png"),
    ("WirelessSprites/txOff.png", "textures/block/transmitter_off.png"),
    ("WirelessSprites/clockerSideOn.png", "textures/block/clocker_on.png"),
    ("WirelessSprites/clockerSideOff.png", "textures/block/clocker_off.png"),
    ("WirelessSprites/remote.png", "textures/item/remote.png"),
    ("WirelessSprites/pd.png", "textures/item/power_configurator.png"),
    ("WirelessSprites/sniff.png", "textures/item/sniffer.png"),
    ("WirelessSprites/triang.png", "textures/item/triangulator.png"),
    ("gui/wifi_small.png", "textures/gui/wifi_small.png"),
    ("gui/wifi_medium.png", "textures/gui/wifi_medium.png"),
    ("gui/wifi_large.png", "textures/gui/wifi_large.png"),
    ("gui/wifiSniff.png", "textures/gui/sniffer.png"),
    ("gui/guiOn.png", "textures/gui/button_on.png"),
    ("gui/guiOff.png", "textures/gui/button_off.png"),
]

LANG = """tile.wirelessredstone.receiver.name=Wireless Receiver
tile.wirelessredstone.transmitter.name=Wireless Transmitter
tile.wirelessredstone.clocker.name=Wireless Clocker
item.wirelessredstone.remote.name=Wireless Remote
item.wirelessredstone.power_configurator.name=Power Configurator
item.wirelessredstone.sniffer.name=Wireless Sniffer
item.wirelessredstone.triangulator.name=Wireless Triangulator
wirelessredstone.gui.frequency=Frequency
wirelessredstone.gui.clock=Clocking Frequency
wirelessredstone.gui.pulse=Pulse
wirelessredstone.gui.direction=Power Direction
wirelessredstone.gui.indirect=Indirect Power
"""


def zips(args: list[str]) -> list[Path]:
    found = []
    for arg in args:
        path = Path(arg)
        found.extend(sorted(path.glob("*.zip")) if path.is_dir() else [path])
    return found


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 1

    wanted = dict(FILES)
    for path in zips(sys.argv[1:]):
        with zipfile.ZipFile(path) as jar:
            for name in jar.namelist():
                target = wanted.pop(name, None)
                if target is None:
                    continue
                out = TARGET / target
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_bytes(jar.read(name))

    if wanted:
        print("missing from the given zips: " + ", ".join(sorted(wanted)), file=sys.stderr)
        return 1

    lang = TARGET / "lang" / "en_US.lang"
    lang.parent.mkdir(parents=True, exist_ok=True)
    lang.write_text(LANG)
    print(f"extracted {len(FILES)} files into {TARGET}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
