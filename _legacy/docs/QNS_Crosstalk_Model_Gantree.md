# QNS v2.4: Crosstalk Symbiosis Model - Gantree Design

> **Document Info**
>
> - **Target Version**: v2.4.0
> - **Focus**: Crosstalk-Aware Compilation & Routing
> - **Format**: Gantree (PPR System)

## 🌳 Gantree Structure

```gantree
QNS_Crosstalk_Model_v2.4 // Root node for v2.4 upgrade (설계중)
    Core_Architecture // Data structures and core traits (설계중)
        CrosstalkMatrix // NxN interaction matrix def (설계중)
            DefineStruct // Struct mapping spectator qubits to error rates (설계중)
            Serialization // JSON/TOML serde impl (설계중)
        HardwareProfile_Ext // Extend existing HardwareProfile (설계중)
            AddCrosstalkField // Add crosstalk_matrix field (설계중)
            UpdateLoadLogic // Parse backend properties for crosstalk (설계중)

    Profiler_Engine // Monitor and characterization (설계중)
        DriftScanner_v2 // Upgrade scanner for simultaneous RB (설계중)
            FetchBackendProperties // Get backend.properties() (설계중)
            ExtractSpectatorError // Parse 'zz_interaction' or similar (설계중)
            MatrixBuilder // Construct CrosstalkMatrix from raw data (설계중)
    
    Rewire_Engine // Optimization core updates (설계중)
        Scoring_Model_v2 // Enhanced fidelity estimation (설계중)
            ParallelGatePenalty // Score penalty for simultaneous neighboring gates (설계중)
            FidelityEstimator_Ext // Update formula with crosstalk term (설계중)
        Router_Optimization // NoiseAwareRouter upgrade (설계중)
            CostFunction_v2 // Heuristic: dist + error + crosstalk (설계중)
            LookaheadHeuristic // Check next-layer gates for potential conflict (설계중)
            
    Simulation_Layer // Verification environment (설계중)
        NoisySimulator_v2 // Add crosstalk noise channel (설계중)
            CrosstalkChannel // Apply error to spectator qubits on 2Q gates (설계중)
            TestCircuitGen // Generate patterns susceptible to crosstalk (설계중)
```

## 📝 Design Logic (PPR Context)

### 1. Atomic Node Analysis

- **DefineStruct**: atomic (Define Rust struct with `HashMap<(Qubit, Qubit), f64>`).
- **ExtractSpectatorError**: atomic (Parse IBM backend property dict).
- **FidelityEstimator_Ext**: atomic (Add `1 - crosstalk_rate` term to existing math).
- **CrosstalkChannel**: atomic (Implement `apply_noise` for spectator indices).

### 2. Decomposition Strategy

- The structure is kept within 4 levels deep to maintain clarity.
- `Router_Optimization` might become complex, but currently fits as sub-modules of Rewire logic.

### 3. Implementation Priority

1. **Core_Architecture**: Essential for data representation.
2. **Profiler_Engine**: Needed to populate the data.
3. **Simulation_Layer**: Needed to verify effect before full router logic.
4. **Rewire_Engine**: The most complex part, dependent on previous steps.

## 📅 Execution Plan (Gantree)

```gantree
Execution_Plan_v2.4 // Sequential implementation steps (설계중)
    Phase_1_Data_Foundation // Core structs and data fetching (완료)
        Implement_CrosstalkMatrix // Rust struct & serialization (완료)
        Update_HardwareProfile // Add field & load logic (완료)
        Implement_DriftScanner_v2 // Fetch & parse IBM backend properties (완료)
        Unit_Test_Data_Layer // Verify parsing correctness (완료)

    Phase_2_Simulation_Env // Verify noise visualizer (완료)
        Implement_CrosstalkChannel // Noise mode logic (완료)
        Update_NoisySimulator // Apply channel to simulator (완료)
        Verify_Simulation_Effect // Compare ideal vs noisy outcomes (완료)

    Phase_3_Optimization_Logic // Core rewiring algorithm (완료)
        Update_FidelityEstimator // Add crosstalk term to math (완료)
        Implement_ParallelGatePenalty // Scoring update (완료)
        Update_LiveRewirer // Logic upgrade (완료) // Note: NoiseAwareRouter IS the core or LiveRewirer backend
        Benchmark_Comparison // Measure fidelity gain (완료)vs v2.3 (대기중)
        
    Phase_4_Integration // Final polish (대기중)
        Update_CLI // Add --crosstalk-aware flag (대기중)
        Documentation // Update QNS_Technical_Specification_v2.4 (대기중)
```
