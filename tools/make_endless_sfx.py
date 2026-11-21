"""Generate original cartoony feedback sounds for the endless demo."""
import math
from pathlib import Path
import random
import struct
import wave

OUTPUT = Path(__file__).resolve().parents[1] / "assets" / "sfx" / "endless"
RATE = 22050

def tone(frequency, t):
    return math.sin(math.tau * frequency * t)

def render(name, duration, sample):
    rng = random.Random(2049)
    pcm = bytearray()
    for i in range(int(RATE * duration)):
        t = i / RATE
        envelope = min(t / .008, (duration - t) / .025, 1)
        value = max(-1, min(1, sample(t, rng.uniform(-1, 1)) * envelope))
        pcm.extend(struct.pack("<h", round(value * 14500)))
    with wave.open(str(OUTPUT / (name + ".wav")), "wb") as out:
        out.setparams((1, 2, RATE, 0, "NONE", "not compressed"))
        out.writeframes(pcm)

OUTPUT.mkdir(parents=True, exist_ok=True)
render("empty", .08, lambda t, n: (n * .23 + tone(240, t) * .25) * math.exp(-45*t))
render("airhorn", .36, lambda t, n: tone(345, t)*.55 + tone(507, t)*.28 + tone(1035, t)*.13)
render("confetti", .22, lambda t, n: n*.6*math.exp(-15*t) + tone(530-350*t, t)*.32*math.exp(-12*t))
render("drone", .49, lambda t, n: tone(122+140*t, t)*.35 + tone(260+270*t, t)*.13)
render("sugar_warning", .66, lambda t, n: tone(870, t)*.49 if t % .22 < .11 else 0)
render("hit", .13, lambda t, n: (tone(580, t)*.44 + tone(770, t)*.19) * math.exp(-13*t))
