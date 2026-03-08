"""
registrar.py — Posts this module's manifest to the FuzzyPeanut shell registry.

Run this once on container startup (see docker-compose.yml command).
The registry stores registrations with a TTL; modules must re-register on restart.
"""

import json
import os
import time
import logging
import httpx

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
log = logging.getLogger(__name__)

REGISTRY_URL = os.environ.get("REGISTRY_URL", "http://shell-registry:3100")
MODULE_URL    = os.environ.get("MODULE_URL",   "http://fpnotes-ui:80")
RETRIES       = int(os.environ.get("REGISTRAR_RETRIES", "10"))
RETRY_DELAY   = float(os.environ.get("REGISTRAR_RETRY_DELAY", "3.0"))

with open("manifest.json") as f:
    manifest = json.load(f)

# Override remoteEntry with the runtime URL so the shell knows where to fetch it.
manifest["remoteEntry"] = f"{MODULE_URL}/remoteEntry.js"


def register() -> None:
    for attempt in range(1, RETRIES + 1):
        try:
            resp = httpx.post(f"{REGISTRY_URL}/register", json=manifest, timeout=5.0)
            resp.raise_for_status()
            log.info("Registered %s with shell registry", manifest["id"])
            return
        except Exception as exc:
            log.warning("Registration attempt %d/%d failed: %s", attempt, RETRIES, exc)
            if attempt < RETRIES:
                time.sleep(RETRY_DELAY)

    log.error("Could not register with shell registry after %d attempts — giving up", RETRIES)


if __name__ == "__main__":
    register()
