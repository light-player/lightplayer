//! Common helper functions for instruction conversion.

use crate::backend::transform::fixed32::types::FixedPointFormat;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Map an old value to its new equivalent.
///
/// If the value is not in the map, returns the value unchanged.
pub fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
}

/// Map a value through the value map (alias for consistency with existing code).
pub fn map_operand(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    map_value(value_map, old_value)
}

/// Extract binary operands from an instruction.
///
/// Returns an error if the instruction is not in Binary format.
pub fn extract_binary_operands(
    old_func: &Function,
    old_inst: Inst,
) -> Result<(Value, Value), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Binary { args, .. } = inst_data {
        Ok((args[0], args[1]))
    } else {
        Err(GlslError::new(
            ErrorCode::E0301,
            alloc::format!(
                "Expected binary instruction format, got: {:?} (opcode: {:?})",
                inst_data,
                old_func.dfg.insts[old_inst].opcode()
            ),
        ))
    }
}

/// Extract unary operand from an instruction.
///
/// Returns an error if the instruction is not in Unary format.
pub fn extract_unary_operand(old_func: &Function, old_inst: Inst) -> Result<Value, GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Unary { arg, .. } = inst_data {
        Ok(*arg)
    } else {
        Err(GlslError::new(
            ErrorCode::E0301,
            alloc::format!(
                "Expected unary instruction format, got: {:?} (opcode: {:?})",
                inst_data,
                old_func.dfg.insts[old_inst].opcode()
            ),
        ))
    }
}

/// Get the first result value from an instruction.
pub fn get_first_result(old_func: &Function, old_inst: Inst) -> Value {
    old_func.dfg.first_result(old_inst)
}

/// Create a zero constant for the target fixed-point type.
pub fn create_zero_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    builder.ins().iconst(target_type, 0)
}
