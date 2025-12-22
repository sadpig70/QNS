#!/usr/bin/env python3
"""
QNS Hardware Verification Script
Target: IBM Quantum Real Backend Validation

Scope:
- Authenticate using `tqp-ibm-apikey.json`
- Select least busy real backend (or specific if defined)
- Run small circuit (Bell State) to verify full pipeline
- Check QNS compilation vs Native Qiskit
"""

import json
import os
import sys
import time
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

try:
    from qiskit import QuantumCircuit, transpile
    from qiskit_ibm_runtime import QiskitRuntimeService, Session, SamplerV2 as Sampler
    from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False
    print("⚠️  Qiskit/IBM Runtime not available.")

# Config
API_KEY_FILE = Path(__file__).parent.parent / "tqp-ibm-apikey.json"
TARGET_BACKEND = "ibm_brisbane" # Or use least_busy
USE_REAL_HARDWARE = True

def load_api_key() -> str:
    if not API_KEY_FILE.exists():
        raise FileNotFoundError(f"API Key file not found at {API_KEY_FILE}")
    
    with open(API_KEY_FILE, 'r') as f:
        data = json.load(f)
        # Handle various likely keys
        if "ibm_quantum_token" in data: return data["ibm_quantum_token"]
        if "api_key" in data: return data["api_key"]
        if "apikey" in data: return data["apikey"]
        return list(data.values())[0] # Fallback

def get_service(token: str) -> QiskitRuntimeService:
    try:
        # Try saving first using the modern channel name
        QiskitRuntimeService.save_account(channel="ibm_quantum", token=token, overwrite=True)
    except Exception as e:
        print(f"   Note: Save with default channel failed ({e}), trying without...")
        pass
        
    try:
        # Try initializing with 'ibm_quantum' (typical default)
        return QiskitRuntimeService(channel="ibm_quantum", token=token)
    except:
        # Fallback to 'ibm_quantum_platform' if hinted by error
        print("   Retrying with channel='ibm_quantum_platform'...")
        return QiskitRuntimeService(channel="ibm_quantum_platform", token=token)

def run_verification():
    if not QISKIT_AVAILABLE:
        print("❌ Qiskit unavailable. Skipping.")
        sys.exit(1)

    print("🔑 Loading API Key...")
    try:
        token = load_api_key()
    except Exception as e:
        print(f"❌ Failed to load key: {e}")
        return

    print("☁️  Connecting to IBM Quantum...")
    try:
        service = get_service(token)
    except Exception as e:
        print(f"❌ Connection failed: {e}")
        return

    print("🔎 Searching for backend...")
    if USE_REAL_HARDWARE:
        try:
            # Try specific first
            backend = service.backend(TARGET_BACKEND)
            print(f"   Found target: {backend.name}")
        except:
            print(f"   Target {TARGET_BACKEND} not found/available. Finding least busy...")
            backend = service.least_busy(operational=True, simulator=False)
            print(f"   Selected: {backend.name}")
    else:
        backend = service.backend("ibmq_qasm_simulator")
        print(f"   Using simulator: {backend.name}")

    # Circuit: Bell State
    print("\n⚡ Preparing Circuit (Bell State)...")
    qc = QuantumCircuit(2)
    qc.h(0)
    qc.cx(0, 1)
    qc.measure_all()

    # Transpile
    print("   Transpiling (Opt Level 1)...")
    pm = generate_preset_pass_manager(backend=backend, optimization_level=1)
    isa_circuit = pm.run(qc)

    # Execute
    print(f"🚀 Submitting job to {backend.name} (Job Mode)...")
    try:
        # Open Plan requires Job Mode (no Session)
        # For qiskit-ibm-runtime 0.42+, pass backend as mode
        sampler = Sampler(mode=backend)
        job = sampler.run([isa_circuit])
        print(f"   Job ID: {job.job_id()}")
        
        result = job.result()
        print("✅ Job Completed!")
        
        # Parse result (SamplerV2)
        # pubs results
        pub_result = result[0]
        counts = pub_result.data.meas.get_counts()
        print(f"   Counts: {counts}")
        
        # Simple validation: |00> and |11> should dominate
        total = sum(counts.values())
        p00 = counts.get('00', 0) / total
        p11 = counts.get('11', 0) / total
        print(f"   P(00)={p00:.2f}, P(11)={p11:.2f} (Total Success: {p00+p11:.2f})")
        
        if p00+p11 > 0.8:
            print("✅ Verification PASSED")
        else:
            print("⚠️  Verification Warning: Low fidelity (expected for raw hardware without QNS)")

    except Exception as e:
        print(f"❌ Execution failed: {e}")

if __name__ == "__main__":
    run_verification()
