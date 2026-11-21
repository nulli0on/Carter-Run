"""Local court speech. Captures audio in memory and returns only editable text."""
import json
import os
from pathlib import Path
import sys
import time


def listen():
    import sounddevice
    import vosk

    path = Path(os.environ.get("CARTEROS_VOSK_MODEL", "speech/vosk-model-small-en-us-0.15"))
    if not path.is_dir():
        raise RuntimeError("Install the local Vosk model first")
    vosk.SetLogLevel(-1)
    model = vosk.Model(str(path))
    device = sounddevice.query_devices(kind="input")
    rate = int(device["default_samplerate"])
    recognizer = vosk.KaldiRecognizer(model, rate)
    block = max(rate // 8, 1)
    words = []
    with sounddevice.RawInputStream(samplerate=rate, blocksize=block, dtype="int16", channels=1) as stream:
        print("READY", flush=True)
        end = time.monotonic() + 7
        while time.monotonic() < end:
            samples, _ = stream.read(block)
            if recognizer.AcceptWaveform(bytes(samples)):
                words.append(json.loads(recognizer.Result())["text"])
    words.append(json.loads(recognizer.FinalResult())["text"])
    print("DRAFT " + " ".join(" ".join(words).split())[:145], flush=True)


if __name__ == "__main__":
    try:
        if sys.argv[1:] == ["listen"]:
            listen()
        else:
            raise ValueError("Expected listen")
    except (OSError, RuntimeError, ValueError, ImportError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
