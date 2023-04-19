use std::{error::Error, marker::PhantomData};

use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{ConstraintSystem, TableColumn},
};
use halo2curves::FieldExt;

use crate::{circuit::CircuitError, fieldutils::i128_to_felt, tensor::Tensor};

use super::LookupOp;

/// Halo2 lookup table for element wise non-linearities.
// Table that should be reused across all lookups (so no Clone)
#[derive(Clone, Debug)]
pub struct Table<F: FieldExt> {
    /// composed operations represented by the table
    pub nonlinearity: LookupOp,
    /// Input to table.
    pub table_input: TableColumn,
    /// Output of table
    pub table_output: TableColumn,
    /// Flags if table has been previously assigned to.
    pub is_assigned: bool,
    /// Number of bits used in lookup table.
    pub bits: usize,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> Table<F> {
    /// Configures the table.
    pub fn configure(
        cs: &mut ConstraintSystem<F>,
        bits: usize,
        nonlinearity: &LookupOp,
    ) -> Table<F> {
        Table {
            nonlinearity: nonlinearity.clone(),
            table_input: cs.lookup_table_column(),
            table_output: cs.lookup_table_column(),
            is_assigned: false,
            bits,
            _marker: PhantomData,
        }
    }
    /// Assigns values to the constraints generated when calling `configure`.
    pub fn layout(&mut self, layouter: &mut impl Layouter<F>) -> Result<(), Box<dyn Error>> {
        if self.is_assigned {
            return Err(Box::new(CircuitError::TableAlreadyAssigned));
        }

        let base = 2i128;
        let smallest = -base.pow(self.bits as u32 - 1);
        let largest = base.pow(self.bits as u32 - 1);
        // let smallest = -base.pow(3);
        // let largest = base.pow(3);
        let inputs = Tensor::from(smallest..largest);
        // println!("Are we here Tuesday input {:?}", inputs);
        let evals = self.nonlinearity.f(inputs.clone())?;
        // println!("Tuesday If we are here then evals {:?}", evals);

        self.is_assigned = true;
        layouter
            .assign_table(
                || "nl table",
                |mut table| {
                    let _ = inputs
                        .iter()
                        .enumerate()
                        .map(|(row_offset, input)| {
                            table.assign_cell(
                                || format!("nl_i_col row {}", row_offset),
                                self.table_input,
                                row_offset,
                                || Value::known(i128_to_felt::<F>(*input)),
                            )?;

                            table.assign_cell(
                                || format!("nl_o_col row {}", row_offset),
                                self.table_output,
                                row_offset,
                                || Value::known(i128_to_felt::<F>(evals[row_offset])),
                            )?;
                            // println!("All good here inside assign table, Tuesday");
                            Ok(())
                        })
                        .collect::<Result<Vec<()>, halo2_proofs::plonk::Error>>()?;
                    Ok(())
                },
            )
            .map_err(Box::<dyn Error>::from)
    }
}
