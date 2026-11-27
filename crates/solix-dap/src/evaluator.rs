//! Expression evaluator for Watch window, Hover tooltips, and Conditional Breakpoints.

#![forbid(unsafe_code)]

use solix_runtime::value::Value;
use solix_runtime::vm::engine::VirtualMachine;

/// Strips matching balanced outer parentheses from an expression string.
fn strip_enclosing_parentheses(input_text: &str) -> &str {
    let mut current_slice = input_text.trim();
    while current_slice.starts_with('(') && current_slice.ends_with(')') {
        let mut nesting_depth = 0i32;
        let mut matched_outermost = false;
        for (byte_index, character) in current_slice.char_indices() {
            if character == '(' {
                nesting_depth += 1;
            } else if character == ')' {
                nesting_depth -= 1;
                if nesting_depth == 0 {
                    if byte_index == current_slice.len() - 1 {
                        matched_outermost = true;
                    }
                    break;
                }
            }
        }
        if matched_outermost {
            current_slice = current_slice[1..current_slice.len() - 1].trim();
        } else {
            break;
        }
    }
    current_slice
}

/// Finds the byte offset of an operator token occurring strictly at parenthesis depth zero.
fn find_operator_at_zero_depth(
    input_text: &str,
    operator_token: &str,
    from_right: bool,
) -> Option<usize> {
    let mut nesting_depth = 0i32;
    if from_right {
        for (byte_index, character) in input_text.char_indices().rev() {
            if character == ')' {
                nesting_depth += 1;
            } else if character == '(' {
                nesting_depth -= 1;
            } else if nesting_depth == 0 && input_text[byte_index..].starts_with(operator_token) {
                return Some(byte_index);
            }
        }
    } else {
        for (byte_index, character) in input_text.char_indices() {
            if character == '(' {
                nesting_depth += 1;
            } else if character == ')' {
                nesting_depth -= 1;
            } else if nesting_depth == 0 && input_text[byte_index..].starts_with(operator_token) {
                return Some(byte_index);
            }
        }
    }
    None
}

/// Evaluates a user expression in the context of an active stack frame.
pub fn evaluate_expression_in_frame(
    expression_text: &str,
    virtual_machine: &VirtualMachine,
    frame_index: usize,
) -> Result<Value, String> {
    let stripped_expression = strip_enclosing_parentheses(expression_text);
    let trimmed_expression = stripped_expression.trim();
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
    if let Some(stripped_long) = trimmed_expression
        .strip_suffix('L')
        .or_else(|| trimmed_expression.strip_suffix('l'))
    {
        if let Ok(long_value) = stripped_long.trim().parse::<i64>() {
            return Ok(Value::I64(long_value));
        }
    }
    if let Ok(float_value) = trimmed_expression.parse::<f64>() {
        return Ok(Value::F64(float_value));
    }

    // Check logical disjunction (||) at depth zero with short-circuiting
    if let Some(operator_position) = find_operator_at_zero_depth(trimmed_expression, "||", false) {
        let left_operand_text = &trimmed_expression[..operator_position];
        let right_operand_text = &trimmed_expression[operator_position + 2..];
        let left_evaluated_value =
            evaluate_expression_in_frame(left_operand_text, virtual_machine, frame_index)?;
        return match left_evaluated_value {
            Value::Bool(true) => Ok(Value::Bool(true)),
            Value::Bool(false) => {
                let right_evaluated_value =
                    evaluate_expression_in_frame(right_operand_text, virtual_machine, frame_index)?;
                match right_evaluated_value {
                    Value::Bool(right_boolean) => Ok(Value::Bool(right_boolean)),
                    right_value => Err(format!(
                        "Logical operator '||' requires boolean operands, found 'bool' and '{}'",
                        right_value.type_name()
                    )),
                }
            }
            left_value => Err(format!(
                "Logical operator '||' requires boolean operands, found '{}'",
                left_value.type_name()
            )),
        };
    }

    // Check logical conjunction (&&) at depth zero with short-circuiting
    if let Some(operator_position) = find_operator_at_zero_depth(trimmed_expression, "&&", false) {
        let left_operand_text = &trimmed_expression[..operator_position];
        let right_operand_text = &trimmed_expression[operator_position + 2..];
        let left_evaluated_value =
            evaluate_expression_in_frame(left_operand_text, virtual_machine, frame_index)?;
        return match left_evaluated_value {
            Value::Bool(false) => Ok(Value::Bool(false)),
            Value::Bool(true) => {
                let right_evaluated_value =
                    evaluate_expression_in_frame(right_operand_text, virtual_machine, frame_index)?;
                match right_evaluated_value {
                    Value::Bool(right_boolean) => Ok(Value::Bool(right_boolean)),
                    right_value => Err(format!(
                        "Logical operator '&&' requires boolean operands, found 'bool' and '{}'",
                        right_value.type_name()
                    )),
                }
            }
            left_value => Err(format!(
                "Logical operator '&&' requires boolean operands, found '{}'",
                left_value.type_name()
            )),
        };
    }

    // Check binary comparison operators (==, !=, <=, >=, <, >) at depth zero BEFORE unary ! / -
    for comparison_operator in &["==", "!=", "<=", ">=", "<", ">"] {
        if let Some(operator_position) =
            find_operator_at_zero_depth(trimmed_expression, comparison_operator, false)
        {
            let left_operand_text = &trimmed_expression[..operator_position];
            let right_operand_text =
                &trimmed_expression[operator_position + comparison_operator.len()..];
            let left_evaluated_value =
                evaluate_expression_in_frame(left_operand_text, virtual_machine, frame_index)?;
            let right_evaluated_value =
                evaluate_expression_in_frame(right_operand_text, virtual_machine, frame_index)?;

            let comparison_result = evaluate_binary_comparison(
                &left_evaluated_value,
                comparison_operator,
                &right_evaluated_value,
            )?;
            return Ok(Value::Bool(comparison_result));
        }
    }

    // Check low-precedence binary arithmetic operators (+, -) at depth zero
    for arithmetic_operator in &["+", "-"] {
        if *arithmetic_operator == "-" && trimmed_expression.starts_with('-') {
            continue;
        }
        if let Some(operator_position) =
            find_operator_at_zero_depth(trimmed_expression, arithmetic_operator, true)
        {
            let left_operand_text = &trimmed_expression[..operator_position];
            let right_operand_text =
                &trimmed_expression[operator_position + arithmetic_operator.len()..];
            if !left_operand_text.trim().is_empty() && !right_operand_text.trim().is_empty() {
                let left_evaluated_value =
                    evaluate_expression_in_frame(left_operand_text, virtual_machine, frame_index)?;
                let right_evaluated_value =
                    evaluate_expression_in_frame(right_operand_text, virtual_machine, frame_index)?;

                return evaluate_binary_arithmetic(
                    &left_evaluated_value,
                    arithmetic_operator,
                    &right_evaluated_value,
                );
            }
        }
    }

    // Check high-precedence binary arithmetic operators (*, /, %) at depth zero
    for arithmetic_operator in &["*", "/", "%"] {
        if let Some(operator_position) =
            find_operator_at_zero_depth(trimmed_expression, arithmetic_operator, true)
        {
            let left_operand_text = &trimmed_expression[..operator_position];
            let right_operand_text =
                &trimmed_expression[operator_position + arithmetic_operator.len()..];
            if !left_operand_text.trim().is_empty() && !right_operand_text.trim().is_empty() {
                let left_evaluated_value =
                    evaluate_expression_in_frame(left_operand_text, virtual_machine, frame_index)?;
                let right_evaluated_value =
                    evaluate_expression_in_frame(right_operand_text, virtual_machine, frame_index)?;

                return evaluate_binary_arithmetic(
                    &left_evaluated_value,
                    arithmetic_operator,
                    &right_evaluated_value,
                );
            }
        }
    }

    // Check unary logical negation (!) and unary arithmetic negation (-)
    if let Some(stripped_not) = trimmed_expression.strip_prefix('!') {
        let operand_value =
            evaluate_expression_in_frame(stripped_not, virtual_machine, frame_index)?;
        return match operand_value {
            Value::Bool(boolean_value) => Ok(Value::Bool(!boolean_value)),
            other_value => Err(format!(
                "Unary operator '!' requires boolean operand, found '{}'",
                other_value.type_name()
            )),
        };
    }

    if let Some(stripped_neg) = trimmed_expression.strip_prefix('-') {
        let operand_value =
            evaluate_expression_in_frame(stripped_neg, virtual_machine, frame_index)?;
        return match operand_value {
            Value::I32(int_val) => Ok(Value::I32(-int_val)),
            Value::I64(long_val) => Ok(Value::I64(-long_val)),
            Value::F64(float_val) => Ok(Value::F64(-float_val)),
            other_value => Err(format!(
                "Unary operator '-' requires numeric operand, found '{}'",
                other_value.type_name()
            )),
        };
    }

    // Check member access (e.g. `player.position.x` or `this.field`) right-to-left for chaining
    if let Some(dot_position) = find_operator_at_zero_depth(trimmed_expression, ".", true) {
        let object_expression = &trimmed_expression[..dot_position];
        let field_identifier = trimmed_expression[dot_position + 1..].trim();
        let target_object_value =
            evaluate_expression_in_frame(object_expression, virtual_machine, frame_index)?;

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
        return Err(format!(
            "Cannot read field '{field_identifier}' on non-object"
        ));
    }

    // Check array indexing (e.g. `items[0]` or `matrix[row]`) at depth zero
    if trimmed_expression.ends_with(']') {
        if let Some(open_bracket_index) = find_matching_open_bracket(trimmed_expression) {
            let target_array_text = trimmed_expression[..open_bracket_index].trim();
            let index_expression_text =
                trimmed_expression[open_bracket_index + 1..trimmed_expression.len() - 1].trim();
            if !target_array_text.is_empty() && !index_expression_text.is_empty() {
                let target_array_value =
                    evaluate_expression_in_frame(target_array_text, virtual_machine, frame_index)?;
                let index_value = evaluate_expression_in_frame(
                    index_expression_text,
                    virtual_machine,
                    frame_index,
                )?;
                let index_integer = match index_value {
                    Value::I32(val) if val >= 0 => val as usize,
                    Value::I64(val) if val >= 0 => val as usize,
                    Value::I32(_) | Value::I64(_) => {
                        return Err("Array index cannot be negative".to_string());
                    }
                    other => {
                        return Err(format!(
                            "Array index must be an integer, found '{}'",
                            other.type_name()
                        ));
                    }
                };
                if let Value::Ref(array_heap_address) = target_array_value {
                    let array_elements = virtual_machine
                        .inspect_array_elements(array_heap_address)
                        .map_err(|runtime_error| runtime_error.message())?;
                    if index_integer < array_elements.len() {
                        return Ok(array_elements[index_integer].2);
                    }
                    return Err(format!(
                        "Array index out of bounds: index {index_integer} >= length {}",
                        array_elements.len()
                    ));
                }
                return Err(format!(
                    "Cannot index non-array value of type '{}'",
                    target_array_value.type_name()
                ));
            }
        }
    }

    // Check register lookup (e.g. `r0`, `r1`)
    if let Some(stripped_register) = trimmed_expression.strip_prefix('r') {
        if let Ok(register_number) = stripped_register.parse::<u16>() {
            if let Some(target_frame) = virtual_machine.call_frames.get(frame_index) {
                let absolute_register_index =
                    target_frame.register_base as usize + register_number as usize;
                if let Some(register_value) =
                    virtual_machine.register_stack.get(absolute_register_index)
                {
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

    Err(format!(
        "Undefined identifier or symbol '{trimmed_expression}'"
    ))
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
        (Value::I32(left_int), Value::I64(_)) => evaluate_binary_comparison(
            &Value::I64(i64::from(*left_int)),
            operator_string,
            right_value,
        ),
        (Value::I64(_), Value::I32(right_int)) => evaluate_binary_comparison(
            left_value,
            operator_string,
            &Value::I64(i64::from(*right_int)),
        ),
        (Value::I32(left_int), Value::F64(_)) => evaluate_binary_comparison(
            &Value::F64(f64::from(*left_int)),
            operator_string,
            right_value,
        ),
        (Value::F64(_), Value::I32(right_int)) => evaluate_binary_comparison(
            left_value,
            operator_string,
            &Value::F64(f64::from(*right_int)),
        ),
        (Value::I64(left_long), Value::F64(_)) => {
            evaluate_binary_comparison(&Value::F64(*left_long as f64), operator_string, right_value)
        }
        (Value::F64(_), Value::I64(right_long)) => {
            evaluate_binary_comparison(left_value, operator_string, &Value::F64(*right_long as f64))
        }
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
        (Value::F64(left_float), Value::F64(right_float)) => match operator_string {
            "+" => Ok(Value::F64(left_float + right_float)),
            "-" => Ok(Value::F64(left_float - right_float)),
            "*" => Ok(Value::F64(left_float * right_float)),
            "/" => {
                if *right_float == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::F64(left_float / right_float))
                }
            }
            "%" => {
                if *right_float == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::F64(left_float % right_float))
                }
            }
            _ => Err(format!("Unsupported operator '{operator_string}'")),
        },
        (Value::I32(left_int), Value::I64(_)) => evaluate_binary_arithmetic(
            &Value::I64(i64::from(*left_int)),
            operator_string,
            right_value,
        ),
        (Value::I64(_), Value::I32(right_int)) => evaluate_binary_arithmetic(
            left_value,
            operator_string,
            &Value::I64(i64::from(*right_int)),
        ),
        (Value::I32(left_int), Value::F64(_)) => evaluate_binary_arithmetic(
            &Value::F64(f64::from(*left_int)),
            operator_string,
            right_value,
        ),
        (Value::F64(_), Value::I32(right_int)) => evaluate_binary_arithmetic(
            left_value,
            operator_string,
            &Value::F64(f64::from(*right_int)),
        ),
        (Value::I64(left_long), Value::F64(_)) => {
            evaluate_binary_arithmetic(&Value::F64(*left_long as f64), operator_string, right_value)
        }
        (Value::F64(_), Value::I64(right_long)) => {
            evaluate_binary_arithmetic(left_value, operator_string, &Value::F64(*right_long as f64))
        }
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

    if let Some(stripped_condition) = trimmed_condition.strip_prefix("<=") {
        if let Ok(target_number) = stripped_condition.trim().parse::<u64>() {
            return hit_count <= target_number;
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

    if let Some(stripped_condition) = trimmed_condition.strip_prefix('<') {
        if let Ok(target_number) = stripped_condition.trim().parse::<u64>() {
            return hit_count < target_number;
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
                match evaluate_expression_in_frame(&expression_buffer, virtual_machine, frame_index)
                {
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

fn find_matching_open_bracket(expression_text: &str) -> Option<usize> {
    if !expression_text.ends_with(']') {
        return None;
    }
    let mut bracket_depth = 0;
    for (byte_index, character) in expression_text.char_indices().rev() {
        match character {
            ']' => bracket_depth += 1,
            '[' => {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    return Some(byte_index);
                }
            }
            _ => {}
        }
    }
    None
}
