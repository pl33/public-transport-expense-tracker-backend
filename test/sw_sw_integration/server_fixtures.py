# SPDX-License-Identifier: MPL-2.0
#   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import httpx
import os
import pytest
import select
import signal
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from subprocess import Popen, PIPE

from client.api_config import APIConfig


def create_token(tmpdir: Path, key_id: str, subject: str, write: bool):
    token_manager_path = (Path(__file__).parent.parent.parent / "jwt_auth" / "target" / "debug" / "token")
    token_manager_base_args = [
        str(token_manager_path),
        "create-token",
        "-k",
        key_id,
        "--issuer",
        "local",
        "-e",
        time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(time.time() + 86400)),
        "-a",
        "http://localhost:8080",
    ]
    if write:
        token_manager_base_args.append("--claims-json")
        token_manager_base_args.append("{\"ptet:write\":true}")
    token_manager_base_args.append(subject)
    with Popen(
            token_manager_base_args,
            cwd=str(tmpdir.absolute()),
            stdout=PIPE,
            stderr=PIPE,
            preexec_fn=os.setsid,
    ) as proc:
        token = proc.stdout.readline().decode().strip()
    return token


@pytest.fixture
def dut():
    with TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)

        # Copy JWT signing key
        key_id = "p6mnVNKuqM4G8WNL"
        keys_root = tmpdir / "keys"
        keys_root.mkdir(parents=True, exist_ok=True)
        (keys_root / "default.txt").write_text(key_id)
        key_dir = keys_root / f"key_{key_id}"
        key_dir.mkdir(parents=True, exist_ok=True)
        (key_dir / "private.pem").write_text(
            "-----BEGIN PRIVATE KEY-----\n"
            "MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDCPLrdEhzu3ScR\n"
            "Zk3ZmQsjNo0FW4X7VB7FxXkBrAFeozugKPmUPmxZWQuyqxfJd0t1mEoy94XXoJbi\n"
            "ihJNy36aMG8o4wtcOC3t9o+ijpILJEr0lwg3J77pIFYX6mCPWeEeSsa/NGwkX4Wx\n"
            "mfNug2w348OtQ5QoxWgDEL0IkCjXsYUFvSLTGZICkCs8Jb2Il/Oy/vq/8ugnW3tV\n"
            "t7B7AI0yyJA+ALn9Kitmyw2R0ZqFDyVI3Kzcj8SYwdl0dkxVrjMGmt5rOOAnzu0y\n"
            "4/jYMJTeaHS79pqIYH2Lo9wIKjMg58OXYr2LllskedjFg1BmOxnvC4RCyQ6NInb0\n"
            "gdWYxkehAgMBAAECggEAPCCvmeLTCRB12RmJzOvm+joXUorAAYrLUd6s9FsKO4Ed\n"
            "Ypl8lSrzwH7js6XqIagnuFnx0jA0gwkH3E3wl5uAb/vBVW9fY2dStIDoeuJWjFNq\n"
            "Tqf4V5aarzEe5Z1c0dN5cDamqqbwORxG1zE4ncPaOrzrpJWwZiSh650BX4a81fPn\n"
            "1XotDoGqL5TncttrbOurtxucywoGHhFHrbcxnsHZB3sn9jE3cVFyQ6KldA8NX1+Y\n"
            "/Z2Kt1Uj7krKCsdeQjchEYllGnl/64VEjje3iXtLedmuWTjayEbR158a+qmaBWuz\n"
            "yWfs/oINycgwH07OjUK9HT+65MCZSlF1FynLn4ZQNwKBgQD3QmhB//wm7YfSuZMg\n"
            "pi+qgscd32BD6GwxSB79t6h7SVQ6osRTK6OIda2UPClfrDmNM72XpCmPTfQxmzc7\n"
            "eosy7rCURmy2kg9ab4vY73rN1kVMQyEmwpOK+Pep1TmiJ+WBBBvlOKR9hPMsQ7pU\n"
            "WDdHofXDNKITieVc5aCOFLRz0wKBgQDJGn6VEQm2P4MS7YEjiIR6/y2chtgeHxsH\n"
            "UDfQkAFwioDditEfcnpIJmK+IO/AxOwtQcp28lrsfw3hUbei28m5+D/Petromu0H\n"
            "5tsaQnsgcGjCg882V/eIqmvoyW88YjHZOJ86HNxfW2x6lN2F7L+lDQqMCDi7/xJI\n"
            "HVEvEdJSOwKBgB1Nsnt38hNO/VTdB3HMIEQOAbkpmwgRSJlCmeGp3X7W/vOADNJq\n"
            "jpQCllLGGoUrLRrt8d9B5mcEbxdd9NrIuCyOG0FHY2TzxHwMUj+giiZQ6Z+TKR+l\n"
            "2cSTow5upcjw/4Md0IyG/P+fPQ6W7ENIvSxxJmY/G6JDKnqrAC82hvhfAoGBAKxk\n"
            "5wB4s5+lAIMYUO0aGZ+q4vnc6qWfurbhQIes+17zjJbeankP1N3G8jzU1VKmPWRq\n"
            "ktq21dUI8egm+kpFKcAfnOwLAGAId4ufjlILjat4UYX2IosOi+d/WLQdAht8fgkd\n"
            "rfnORegE4pCOzvwAwSUHcfQrFB+tAIoLf83e9DKDAoGBAK/lozkBjXYilvq+P0im\n"
            "MU1cDMivXki/zPm6YQAtA5KebkkVEgiJ67ICMDooKFjTMyRL7OTJmiui8XsXeiun\n"
            "+1KUbamuEHutZkKso1F+Fjz5jHm/UGlCExYhZwGL2TDliWPs8dV8NGc0xz00Xzo2\n"
            "C4MsiYckK8oGAW66IKOpHaaj\n"
            "-----END PRIVATE KEY-----\n"
        )
        (key_dir / "public.pem").write_text(
            "-----BEGIN PUBLIC KEY-----\n"
            "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwjy63RIc7t0nEWZN2ZkL\n"
            "IzaNBVuF+1QexcV5AawBXqM7oCj5lD5sWVkLsqsXyXdLdZhKMveF16CW4ooSTct+\n"
            "mjBvKOMLXDgt7faPoo6SCyRK9JcINye+6SBWF+pgj1nhHkrGvzRsJF+FsZnzboNs\n"
            "N+PDrUOUKMVoAxC9CJAo17GFBb0i0xmSApArPCW9iJfzsv76v/LoJ1t7VbewewCN\n"
            "MsiQPgC5/SorZssNkdGahQ8lSNys3I/EmMHZdHZMVa4zBpreazjgJ87tMuP42DCU\n"
            "3mh0u/aaiGB9i6PcCCozIOfDl2K9i5ZbJHnYxYNQZjsZ7wuEQskOjSJ29IHVmMZH\n"
            "oQIDAQAB\n"
            "-----END PUBLIC KEY-----\n"
        )

        # Start server
        dut_path = (Path(__file__).parent.parent.parent / "target" / "debug" / "public-transport-expense-tracker")
        dut = Popen(
            [
                str(dut_path),
                "--database",
                f"sqlite://{(tmpdir / "db.sqlite3").absolute()}?mode=rwc",
                "--keys-dir",
                str(keys_root.absolute()),
                "-u",
                "http://localhost:8080",
            ],
            cwd=str(tmpdir.absolute()),
            stdout=PIPE,
            stderr=PIPE,
            preexec_fn=os.setsid,
        )

        # Create access tokens for User 1
        read_token_1 = create_token(tmpdir, key_id, "test1@example.tld", False)
        write_token_1 = create_token(tmpdir, key_id, "test1@example.tld", True)

        # Create access tokens for User 2
        read_token_2 = create_token(tmpdir, key_id, "test2@example.tld", False)
        write_token_2 = create_token(tmpdir, key_id, "test2@example.tld", True)

        # Wait for heartbeat
        base_url = "http://localhost:8080/api/v1"
        with httpx.Client(base_url=base_url, verify=True) as client:
            for step in range(10):
                try:
                    response = client.get("/openapi.json")
                    if response.status_code == 200:
                        break
                except:
                    pass
                time.sleep(1.0)

        yield {
            "base_url": base_url,
            "read_token_1": read_token_1,
            "write_token_1": write_token_1,
            "read_token_2": read_token_2,
            "write_token_2": write_token_2,
        }

        pgid = os.getpgid(dut.pid)
        os.killpg(pgid, signal.SIGTERM)
        dut.wait()


@pytest.fixture
def api_config_dict(dut):
    return {
        "read": APIConfig(
            base_path=dut["base_url"],
            access_token=dut["read_token_1"],
        ),
        "read_write": APIConfig(
            base_path=dut["base_url"],
            access_token=dut["write_token_1"],
        ),
        "unauthorized": APIConfig(
            base_path=dut["base_url"],
        ),
        "read_2": APIConfig(
            base_path=dut["base_url"],
            access_token=dut["read_token_2"],
        ),
        "read_write_2": APIConfig(
            base_path=dut["base_url"],
            access_token=dut["write_token_2"],
        ),
    }


@pytest.fixture
def api_config_unauthorized(api_config_dict):
    return api_config_dict["unauthorized"]


@pytest.fixture
def api_config_read(api_config_dict):
    return api_config_dict["read"]


@pytest.fixture
def api_config_readwrite(api_config_dict):
    return api_config_dict["read_write"]
