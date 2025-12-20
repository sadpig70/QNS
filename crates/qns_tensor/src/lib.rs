use ndarray::{Array2, Array3};
use num_complex::Complex64;
use qns_core::types::Gate;
use qns_core::{QnsError, Result};
use qns_noise::NoiseChannel;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TensorError {
    #[error("Dimension mismatch: expected {0}, got {1}")]
    DimensionMismatch(String, String),
    #[error("Contraction error: {0}")]
    ContractionError(String),
}

/// Matrix Product State (MPS) representation of a quantum state.
pub struct TensorNetwork {
    num_qubits: usize,
    /// MPS tensors. Each tensor is rank-3: (left_bond, physical, right_bond).
    /// For boundaries, bond dimension is 1.
    nodes: Vec<Array3<Complex64>>,
    /// Maximum bond dimension (chi)
    max_bond_dim: usize,
    /// Noise model to apply
    noise_model: Option<Box<dyn NoiseChannel>>,
}

impl TensorNetwork {
    /// Creates a new TensorNetwork in the |00...0> state.
    pub fn new(num_qubits: usize, max_bond_dim: usize) -> Self {
        let mut nodes = Vec::with_capacity(num_qubits);

        for _i in 0..num_qubits {
            // Initial state |0> for each qubit.
            // Tensor shape: (1, 2, 1)
            // |0> = [1, 0]
            let mut node = Array3::<Complex64>::zeros((1, 2, 1));
            node[[0, 0, 0]] = Complex64::new(1.0, 0.0);
            // node[[0, 1, 0]] is 0.0
            nodes.push(node);
        }

        Self {
            num_qubits,
            nodes,
            max_bond_dim,
            noise_model: None,
        }
    }

    /// Sets the noise model for the simulation.
    pub fn with_noise(mut self, noise_model: Box<dyn NoiseChannel>) -> Self {
        self.noise_model = Some(noise_model);
        self
    }

    /// Returns the number of qubits.
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Applies a gate to the tensor network.
    /// Currently only supports single qubit gates and nearest-neighbor two-qubit gates.
    pub fn apply_gate(&mut self, gate: &Gate) -> Result<()> {
        // 1. Apply the ideal gate
        self.apply_ideal_gate(gate)?;

        // 2. Apply noise if configured
        if let Some(noise_model) = &self.noise_model {
            let error_ops = noise_model.apply(gate);

            // Stochastic Unravelling: Pick one error operator based on probability
            let mut rng = rand::thread_rng();
            use rand::Rng;
            let r: f64 = rng.gen();

            let mut cum_prob = 0.0;
            for (prob, error_gate) in error_ops {
                cum_prob += prob;
                if r < cum_prob {
                    // Apply this error gate
                    // Note: Error gate might be Identity (no error)
                    // We assume error gates are simple (single qubit) for now,
                    // or at least supported by apply_ideal_gate.
                    // Ideally we should log which error occurred for debugging.

                    // Avoid infinite recursion by calling apply_ideal_gate directly
                    self.apply_ideal_gate(&error_gate)?;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Internal method to apply a gate without noise (to avoid recursion).
    fn apply_ideal_gate(&mut self, gate: &Gate) -> Result<()> {
        match gate {
            Gate::H(q)
            | Gate::X(q)
            | Gate::Y(q)
            | Gate::Z(q)
            | Gate::S(q)
            | Gate::T(q)
            | Gate::Rx(q, _)
            | Gate::Ry(q, _)
            | Gate::Rz(q, _) => self.apply_single_qubit_gate(*q, gate),
            Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => {
                if (c.max(t) - c.min(t)) == 1 {
                    self.apply_two_qubit_gate(*c, *t, gate)
                } else {
                    // For MVP, we require SWAP routing first.
                    // Or we could implement SWAP chain here implicitly, but better to use Router.
                    Err(QnsError::Unsupported("Non-nearest neighbor gates not supported in TensorNetwork yet. Use Router first.".to_string()))
                }
            },
            _ => Err(QnsError::Unsupported(format!(
                "Gate {} not supported",
                gate
            ))),
        }
    }

    fn apply_single_qubit_gate(&mut self, qubit: usize, gate: &Gate) -> Result<()> {
        let matrix = gate
            .matrix_2x2()
            .ok_or_else(|| QnsError::Simulator("Failed to get gate matrix".to_string()))?;
        let node = &mut self.nodes[qubit];

        // Contract physical index (axis 1) with gate matrix.
        // Node: (L, P, R), Matrix: (P_out, P_in)
        // New Node: (L, P_out, R)

        let (l, _, r) = node.dim();
        let mut new_node = Array3::<Complex64>::zeros((l, 2, r));

        for i in 0..l {
            for j in 0..r {
                // Extract physical vector for this bond config
                let vec = [node[[i, 0, j]], node[[i, 1, j]]];

                // Multiply matrix * vector
                let res0 = matrix[0][0] * vec[0] + matrix[0][1] * vec[1];
                let res1 = matrix[1][0] * vec[0] + matrix[1][1] * vec[1];

                new_node[[i, 0, j]] = res0;
                new_node[[i, 1, j]] = res1;
            }
        }

        self.nodes[qubit] = new_node;
        Ok(())
    }

    fn apply_two_qubit_gate(&mut self, q1: usize, q2: usize, gate: &Gate) -> Result<()> {
        // Assume q1 and q2 are adjacent. q1 < q2 or q2 < q1.
        let (left_q, right_q) = if q1 < q2 { (q1, q2) } else { (q2, q1) };

        let node_l = &self.nodes[left_q];
        let node_r = &self.nodes[right_q];

        let (dl_l, _, dl_r) = node_l.dim();
        let (dr_l, _, dr_r) = node_r.dim();

        if dl_r != dr_l {
            return Err(QnsError::Simulator(
                "Bond dimension mismatch in MPS".to_string(),
            ));
        }

        // 1. Contract L and R into Theta tensor (dL_L, 2, 2, dR_R)
        // Theta[l, p1, p2, r] = sum_k L[l, p1, k] * R[k, p2, r]
        let mut theta = Array2::<Complex64>::zeros((dl_l * 2 * 2, dr_r));

        // Naive contraction loop (can be optimized)
        for l_idx in 0..dl_l {
            for p1 in 0..2 {
                for p2 in 0..2 {
                    for r_idx in 0..dr_r {
                        let mut sum = Complex64::new(0.0, 0.0);
                        for k in 0..dl_r {
                            sum += node_l[[l_idx, p1, k]] * node_r[[k, p2, r_idx]];
                        }
                        // Flatten indices: (l * 4 + p1 * 2 + p2, r)
                        theta[[l_idx * 4 + p1 * 2 + p2, r_idx]] = sum;
                    }
                }
            }
        }

        // 2. Apply Gate Unitary to Theta
        // Gate is 4x4 matrix acting on (p1, p2)
        // NewTheta[l, p1', p2', r] = sum_{p1,p2} U[p1'p2', p1p2] * Theta[l, p1, p2, r]

        let gate_matrix = gate
            .matrix_4x4()
            .ok_or_else(|| QnsError::Simulator("Failed to get 4x4 gate matrix".to_string()))?;
        let mut new_theta = Array2::<Complex64>::zeros((dl_l * 2 * 2, dr_r));

        for l_idx in 0..dl_l {
            for r_idx in 0..dr_r {
                for p_out in 0..4 {
                    // p1' * 2 + p2'
                    let mut sum = Complex64::new(0.0, 0.0);
                    for p_in in 0..4 {
                        // p1 * 2 + p2
                        // Gate matrix is typically [[row, col]] -> [p_out][p_in]
                        // But check Gate implementation. Usually row-major.
                        sum += gate_matrix[p_out][p_in] * theta[[l_idx * 4 + p_in, r_idx]];
                    }
                    new_theta[[l_idx * 4 + p_out, r_idx]] = sum;
                }
            }
        }

        // 3. SVD: Theta -> U * S * V^dag
        // Reshape Theta to Matrix M: (dl_l * 2) x (2 * dr_r)
        // Rows: (l, p1), Cols: (p2, r)
        let rows = dl_l * 2;
        let cols = 2 * dr_r;

        // Convert to nalgebra DMatrix for SVD
        // nalgebra is column-major by default, but DMatrix::from_row_slice is safer
        // if we organize data row by row.

        // Re-organize data for (l, p1) x (p2, r) matrix
        let mut matrix_data = Vec::with_capacity(rows * cols);
        for r_idx in 0..rows {
            for c_idx in 0..cols {
                // Map (r_idx, c_idx) back to flat theta index
                // r_idx = l * 2 + p1
                // c_idx = p2 * dr_r + r

                let l = r_idx / 2;
                let p1 = r_idx % 2;
                let p2 = c_idx / dr_r;
                let r = c_idx % dr_r;

                let theta_idx = l * 4 + p1 * 2 + p2;
                // theta is (dl_l * 4, dr_r) -> index is theta_idx * dr_r + r ??
                // Wait, theta was Array2 with shape (dl_l * 4, dr_r)
                // Access: theta[[theta_idx, r]]

                matrix_data.push(new_theta[[theta_idx, r]]);
            }
        }

        let m = nalgebra::DMatrix::from_row_slice(rows, cols, &matrix_data);
        let svd = m.svd(true, true); // Compute U and V

        let u = svd.u.unwrap(); // (rows, min(rows, cols))
        let s = svd.singular_values; // (min(rows, cols))
        let v_t = svd.v_t.unwrap(); // (min(rows, cols), cols)

        // 4. Truncate
        // Keep top chi singular values
        let mut chi = self.max_bond_dim;
        let available_chi = s.len();
        if chi > available_chi {
            chi = available_chi;
        }

        // Filter small singular values? (Optional optimization)
        while chi > 1 && s[chi - 1] < 1e-10 {
            chi -= 1;
        }

        // 5. Update Nodes
        // New Left Node: U_trunc * S_trunc (or sqrt(S)) -> Reshape to (dl_l, 2, chi)
        // New Right Node: V_trunc -> Reshape to (chi, 2, dr_r)
        // Standard canonical form: usually put S on the bond or split sqrt(S).
        // Let's put S into Right Node for mixed canonical (or just split).
        // Let's multiply S into V_t.

        // U_trunc: (rows, chi)
        // S_trunc: (chi)
        // V_t_trunc: (chi, cols)

        // New Node L: (dl_l, 2, chi)
        let mut new_node_l = Array3::<Complex64>::zeros((dl_l, 2, chi));
        for i in 0..rows {
            // i = l * 2 + p1
            for k in 0..chi {
                let val = u[(i, k)];
                let l = i / 2;
                let p1 = i % 2;
                new_node_l[[l, p1, k]] = val; // U is unitary, so just copy
            }
        }

        // New Node R: S * V_t -> (chi, cols) -> (chi, 2, dr_r)
        let mut new_node_r = Array3::<Complex64>::zeros((chi, 2, dr_r));
        for k in 0..chi {
            let s_val = Complex64::new(s[k], 0.0);
            for j in 0..cols {
                // j = p2 * dr_r + r
                let val = v_t[(k, j)] * s_val;
                let p2 = j / dr_r;
                let r = j % dr_r;
                new_node_r[[k, p2, r]] = val;
            }
        }

        self.nodes[left_q] = new_node_l;
        self.nodes[right_q] = new_node_r;

        Ok(())
    }

    /// Measure all qubits in the computational basis.
    /// Returns a map of bitstrings to counts.
    /// Note: This is a simplified implementation that contracts the whole network to a state vector first.
    /// For true MPS efficiency, we should sample without full contraction.
    pub fn measure(&self, shots: usize) -> Result<std::collections::HashMap<String, usize>> {
        // Warning: This full contraction is exponential in N.
        // Only feasible for small N (e.g. < 20).
        let state_vector = self.contract_to_state_vector()?;

        let mut counts = std::collections::HashMap::new();
        let mut rng = rand::thread_rng();
        use rand::Rng;

        for _ in 0..shots {
            let r: f64 = rng.gen();
            let mut cum_prob = 0.0;
            let mut selected = 0;

            for (i, amp) in state_vector.iter().enumerate() {
                cum_prob += amp.norm_sqr();
                if r < cum_prob {
                    selected = i;
                    break;
                }
            }

            let bitstring = format!("{:0width$b}", selected, width = self.num_qubits);
            *counts.entry(bitstring).or_insert(0) += 1;
        }

        Ok(counts)
    }

    /// Contract the MPS to a full state vector.
    fn contract_to_state_vector(&self) -> Result<Vec<Complex64>> {
        // Very naive contraction: contract left to right.
        // State starts as (1, 2, 1)
        // Contract with next (1, 2, 1) -> (1, 4, 1)
        // ...

        if self.nodes.is_empty() {
            return Ok(vec![]);
        }

        let current = self.nodes[0].clone();
        let (_, phys, right_bond) = current.dim();
        // Initial shape is (1, 2, right_bond) -> reshape to (2, right_bond)
        // Note: left bond of node 0 is always 1.
        let mut current_vec = current
            .into_shape((phys, right_bond))
            .map_err(|e| QnsError::Simulator(format!("Reshape error: {}", e)))?;

        for i in 1..self.num_qubits {
            let next_node = &self.nodes[i]; // Shape (1, 2, 1) (assuming bond dim 1 for now if no entanglement)
                                            // Actually next_node is (L, 2, R).
                                            // We want to tensor product current_vec (Dim 2^i) with next_node.

            // For this MVP, since we didn't implement proper SVD/Contraction for 2-qubit gates,
            // the nodes might still be effectively product states or simple.
            // But let's write generic logic assuming bond dimensions match.

            // Current: (Dim, Bond)
            // Next: (Bond, 2, NextBond)
            // Result: (Dim * 2, NextBond)

            let (dim, bond) = current_vec.dim();
            let (next_bond, phys, next_next_bond) = next_node.dim();

            if bond != next_bond {
                return Err(QnsError::Simulator(
                    "Bond dimension mismatch during measurement contraction".to_string(),
                ));
            }

            let mut new_vec = Array2::<Complex64>::zeros((dim * 2, next_next_bond));

            for d in 0..dim {
                for b in 0..bond {
                    let val = current_vec[[d, b]];
                    for p in 0..phys {
                        for nb in 0..next_next_bond {
                            let next_val = next_node[[b, p, nb]];
                            // Index in new vec: d * 2 + p
                            new_vec[[d * 2 + p, nb]] += val * next_val;
                        }
                    }
                }
            }
            current_vec = new_vec;
        }

        // Final result should be (2^N, 1). Flatten to Vec.
        Ok(current_vec.iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex64;

    #[test]
    fn test_tensor_network_init() {
        let tn = TensorNetwork::new(3, 2);
        assert_eq!(tn.num_qubits(), 3);

        // Check initial state |000>
        // Node 0: (1, 2, 1) -> [1, 0]
        let node0 = &tn.nodes[0];
        assert_eq!(node0[[0, 0, 0]], Complex64::new(1.0, 0.0));
        assert_eq!(node0[[0, 1, 0]], Complex64::new(0.0, 0.0));
    }

    #[test]
    fn test_apply_single_qubit_gate() {
        let mut tn = TensorNetwork::new(1, 2);
        // Apply X gate to |0> -> |1>
        tn.apply_gate(&Gate::X(0)).unwrap();

        let node = &tn.nodes[0];
        // |1> = [0, 1]
        assert!((node[[0, 0, 0]].re - 0.0).abs() < 1e-10);
        assert!((node[[0, 1, 0]].re - 1.0).abs() < 1e-10);
    }
}
