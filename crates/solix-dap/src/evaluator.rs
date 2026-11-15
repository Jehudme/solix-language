//! Expression evaluator for Watch window, Hover tooltips, and Conditional Breakpoints.

#![forbid(unsafe_code)]

use solix_runtime::value::Value;
use solix_runtime::vm::engine::VirtualMachine;

/// Evaluates a user expression in the context of an active stack frame.
pub fn evaluate_expression_in_frame(
    expression_text: &str,
    virtual_machine: &VirtualMachine,
    frame_index: usize,
) -> Result<Value, String> {
    let trimmed_expression = expression_text.trim();
    if trimmed_expression.is_empty() {
        return Err("Empty expression".to_string());
    }

    // Check boolean literals
    if trimmed_expression == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed_expression == "false" {
        return Ok(Value::Bool(false));
    }
    if trimmed_expression == "null" {
        return Ok(Value::Null);
    }

    // Check integer literals
    if let Ok(integer_value) = trimmed_expression.parse::<i32>() {
        return Ok(Value::I32(integer_value));
    }
    if let Ok(long_value) = trimmed_expression.parse::<i64>() {
        return Ok(Value::I64(long_value));
    }
    if let Ok(float_value) = trimmed_expression.parse::<f64>() {
        return Ok(Value::F64(float_value));
    }

    // Check binary comparison operators (==, !=, <=, >=, <, >)
    for comparison_operator in &["==", "!=", "<=", ">=", "<", ">"] {
        if let Some(operator_position) = trimmed_expression.find(comparison_operator) {
            let left_operand_text = &trimmed_expression[..operator_position];
            let right_operand_text = &trimmed_expression[operator_position + comparison_operator.len()..];
            let left_evaluated_value = evaluate_expression_in_frame(
                left_operand_text,
                virtual_machine,
                frame_index,
            )?;
            let right_evaluated_value = evaluate_expression_in_frame(
                right_operand_text,
                virtual_machine,
                frame_index,
            )?;

            let comparison_result = evaluate_binary_comparison(
                &left_evaluated_value,
                comparison_operator,
                &right_evaluated_value,
            )?;
            return Ok(Value::Bool(comparison_result));
        }
    }

    // Check binary arithmetic operators (+, -, *, /, %)
    for arithmetic_operator in &["+", "-", "*", "/", "%"] {
        // Skip leading negative sign on literals
        if *arithmetic_operator == "-" && trimmed_expression.starts_with('-') {
            continue;
        }
        if let Some(operator_position) = trimmed_expression.rfind(arithmetic_operator) {
            let left_operand_text = &trimmed_expression[..operator_position];
            let right_operand_text = &trimmed_expression[operator_position + arithmetic_operator.len()..];
            if !left_operand_text.trim().is_empty() && !right_operand_text.trim().is_empty() {
                let left_evaluated_value = evaluate_expression_in_frame(
                    left_operand_text,
                    virtual_machine,
                    frame_index,
                )?;
                let right_evaluated_value = evaluate_expression_in_frame(
                    right_operand_text,
                    virtual_machine,
                    frame_index,
                )?;

                return evaluate_binary_arithmetic(
                    &left_evaluated_value,
                    arithmetic_operator,
                    &right_evaluated_value,
                );
            }
        }
    }

    // Check member access (e.g. `origin_point.coordinate_x` or `this.field`)
    if let Some(dot_position) = trimmed_expression.find('.') {
        let object_expression = &trimmed_expression[..dot_position];
        let field_identifier = trimmed_expression[dot_position + 1..].trim();
        let target_object_value = evaluate_expression_in_frame(
            object_expression,
            virtual_machine,
            frame_index,
        )?;

        if let Value::Ref(heap_address) = target_object_value {
            let object_fields = virtual_machine
                .inspect_object_fields(heap_address)
                .map_err(|runtime_error| runtime_error.message())?;
            for (field_name, _field_type, field_val) in object_fields {
                if field_name == field_identifier {
                    return Ok(field_val);
                }
            }
            return Err(format!("Field '{field_identifier}' not found on object"));
        }
        return Err(format!("Cannot read field '{field_identifier}' on non-object"));
    }

    // Check register lookup (e.g. `r0`, `r1`)
    if let Some(stripped_register) = trimmed_expression.strip_prefix('r') {
        if let Ok(register_number) = stripped_register.parse::<u16>() {
            if let Some(target_frame) = virtual_machine.call_frames.get(frame_index) {
                let absolute_register_index = target_frame.register_base + register_number as usize;
                if let Some(register_value) = virtual_machine.register_stack.get(absolute_register_index) {
                    return Ok(*register_value);
                }
            }
        }
    }

    // Identifier lookup in local variables and parameters
    let frame_locals = virtual_machine.inspect_frame_locals(frame_index);
    for (variable_name, _variable_type, variable_value) in frame_locals {
        if variable_name == trimmed_expression {
            return Ok(variable_value);
        }
    }

    // Static variables lookup
    for (static_index, static_slot_value) in virtual_machine.static_slots.iter().enumerate() {
        if format!("static_{static_index}") == trimmed_expression {
            return Ok(*static_slot_value);
        }
    }

    Err(format!("Undefined identifier or symbol '{trimmed_expression}'"))
}

/// Evaluates a binary comparison operator between two runtime values.
fn evaluate_binary_comparison(
    left_value: &Value,
    operator_string: &str,
    right_value: &Value,
) -> Result<bool, String> {
    match (left_value, right_value) {
        (Value::I32(left_int), Value::I32(right_int)) => Ok(match operator_string {
            "==" => left_int == right_int,
            "!=" => left_int != right_int,
            "<" => left_int < right_int,
            "<=" => left_int <= right_int,
            ">" => left_int > right_int,
            ">=" => left_int >= right_int,
            _ => false,
        }),
        (Value::I64(left_long), Value::I64(right_long)) => Ok(match operator_string {
            "==" => left_long == right_long,
            "!=" => left_long != right_long,
            "<" => left_long < right_long,
            "<=" => left_long <= right_long,
            ">" => left_long > right_long,
            ">=" => left_long >= right_long,
            _ => false,
        }),
        (Value::F64(left_float), Value::F64(right_float)) => Ok(match operator_string {
            "==" => left_float == right_float,
            "!=" => left_float != right_float,
            "<" => left_float < right_float,
            "<=" => left_float <= right_float,
            ">" => left_float > right_float,
            ">=" => left_float >= right_float,
            _ => false,
        }),
        (Value::Bool(left_bool), Value::Bool(right_bool)) => Ok(match operator_string {
            "==" => left_bool == right_bool,
            "!=" => left_bool != right_bool,
            _ => false,
        }),
        (Value::Ref(left_ref), Value::Ref(right_ref)) => Ok(match operator_string {
            "==" => left_ref == right_ref,
            "!=" => left_ref != right_ref,
            _ => false,
        }),
        (Value::Null, Value::Ref(right_ref)) => Ok(match operator_string {
            "==" => *right_ref == 0,
            "!=" => *right_ref != 0,
            _ => false,
        }),
        (Value::Ref(left_ref), Value::Null) => Ok(match operator_string {
            "==" => *left_ref == 0,
            "!=" => *left_ref != 0,
            _ => false,
        }),
        (Value::Null, Value::Null) => Ok(match operator_string {
            "==" => true,
            "!=" => false,
            _ => false,
        }),
        _ => Err(format!(
            "Cannot compare values of type '{}' and '{}'",
            left_value.type_name(),
            right_value.type_name()
        )),
    }
}

/// Evaluates a binary arithmetic operator between two runtime values.
fn evaluate_binary_arithmetic(
    left_value: &Value,
    operator_string: &str,
    right_value: &Value,
) -> Result<Value, String> {
    match (left_value, right_value) {
        (Value::I32(left_int), Value::I32(right_int)) => match operator_string {
            "+" => Ok(Value::I32(left_int.wrapping_add(*right_int))),
            "-" => Ok(Value::I32(left_int.wrapping_sub(*right_int))),
            "*" => Ok(Value::I32(left_int.wrapping_mul(*right_int))),
            "/" => {
                if *right_int == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::I32(left_int.wrapping_div(*right_int)))
                }
            }
            "%" => {
                if *right_int == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::I32(left_int.wrapping_rem(*right_int)))
                }
            }
            _ => Err(format!("Unsupported operator '{operator_string}'")),
        },
        (Value::I64(left_long), Value::I64(right_long)) => match operator_string {
            "+" => Ok(Value::I64(left_long.wrapping_add(*right_long))),
            "-" => Ok(Value::I64(left_long.wrapping_sub(*right_long))),
            "*" => Ok(Value::I64(left_long.wrapping_mul(*right_long))),
            "/" => {
                if *right_long == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::I64(left_long.wrapping_div(*right_long)))
                }
            }
            "%" => {
                if *right_long == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::I64(left_long.wrapping_rem(*right_long)))
                }
            }
            _ => Err(format!("Unsupported operator '{operator_string}'")),
        },
        _ => Err(format!(
            "Cannot perform arithmetic on '{}' and '{}'",
            left_value.type_name(),
            right_value.type_name()
        )),
    }
}

/// Tests whether a hit count satisfies a hit condition string (e.g. `5`, `>= 10`, `== 3`, `% 2 == 0`).
#[must_use]
pub fn check_hit_condition(hit_count: u64, condition_text: &str) -> bool {
    let trimmed_condition = condition_text.trim();
    if trimmed_condition.is_empty() {
        return true;
    }

    // Direct number e.g. "5" -> hit_count >= 5
    if let Ok(target_number) = trimmed_condition.parse::<u64>() {
        return hit_count >= target_number;
    }

    if let Some(stripped_condition) = trimmed_condition.strip_prefix(">=") {
        if let Ok(target_number) = stripped_condition.trim().parse::<u64>() {
            return hit_count >= target_number;
        }
    }

    if let Some(stripped_condition) = trimmed_condition.strip_prefix("==") {
        if let Ok(target_number) = stripped_condition.trim().parse::<u64>() {
            return hit_count == target_number;
        }
    }

    if let Some(stripped_condition) = trimmed_condition.strip_prefix('>') {
        if let Ok(target_number) = stripped_condition.trim().parse::<u64>() {
            return hit_count > target_number;
        }
    }

    if let Some(stripped_condition) = trimmed_condition.strip_prefix('%') {
        let modulo_parts: Vec<&str> = stripped_condition.split("==").collect();
        if modulo_parts.len() == 2 {
            if let (Ok(modulo_divisor), Ok(modulo_target)) = (
                modulo_parts[0].trim().parse::<u64>(),
                modulo_parts[1].trim().parse::<u64>(),
            ) {
                if modulo_divisor > 0 {
                    return (hit_count % modulo_divisor) == modulo_target;
                }
            }
        }
    }

    false
}

/// Interpolates `{variable_or_expression}` expressions in a logpoint message template.
pub fn interpolate_log_template(
    template_string: &str,
    virtual_machine: &VirtualMachine,
    frame_index: usize,
) -> String {
    let mut rendered_output = String::new();
    let mut character_iterator = template_string.chars().peekable();

    while let Some(current_character) = character_iterator.next() {
        if current_character == '{' {
            let mut expression_buffer = String::new();
            let mut found_closing_brace = false;

            for next_character in character_iterator.by_ref() {
                if next_character == '}' {
                    found_closing_brace = true;
                    break;
                }
                expression_buffer.push(next_character);
            }

            if found_closing_brace {
                match evaluate_expression_in_frame(
                    &expression_buffer,
                    virtual_machine,
                    frame_index,
                ) {
                    Ok(evaluated_value) => {
                        rendered_output.push_str(&format!("{evaluated_value:?}"));
                    }
                    Err(evaluation_error) => {
                        rendered_output.push_str(&format!("<error: {evaluation_error}>"));
                    }
                }
            } else {
                rendered_output.push('{');
                rendered_output.push_str(&expression_buffer);
            }
        } else {
            rendered_output.push(current_character);
        }
    }

    rendered_output
}
