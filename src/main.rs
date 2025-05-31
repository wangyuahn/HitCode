use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("请提供一个.hc文件");
        return;
    }
    let filename = &args[1];
    let file = File::open(filename).expect("无法打开文件");
    let reader = BufReader::new(file);

    // 类型枚举
    #[derive(Clone, Debug, PartialEq)]
    enum VarType { Int, Double, Str, Bool, Unknown }
    fn parse_type(s: &str) -> VarType {
        match s.trim() {
            "int" => VarType::Int,
            "double" => VarType::Double,
            "str" => VarType::Str,
            "bool" => VarType::Bool,
            _ => VarType::Unknown,
        }
    }

    // 变量/常量类型表
    let mut var_types: HashMap<String, VarType> = HashMap::new();
    let mut const_types: HashMap<String, VarType> = HashMap::new();
    let mut variables: HashMap<String, String> = HashMap::new();
    let mut constants: HashMap<String, String> = HashMap::new();
    let mut lists: HashMap<String, Vec<String>> = HashMap::new();
    let mut functions: HashMap<String, Vec<String>> = HashMap::new();
    // 新增：函数签名和返回值表
    let mut function_sigs: HashMap<String, (Vec<(String, String)>, Option<String>)> = HashMap::new();
    let mut function_returns: HashMap<String, String> = HashMap::new();
    let mut start_commands: Vec<String> = Vec::new();
    let mut current_function: Option<String> = None;
    let mut in_start_section = false;

    for line in reader.lines() {
        let line = line.expect("读取行失败");
        let raw_line = line.clone();
        let line = line.trim_end();
        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') { continue; }
        // 支持函数定义带参数和返回值 function 名(类型 参数, ...)->return::类型
        if line.starts_with("function ") {
            let def = line[9..].trim_end_matches(':').trim();
            let (name_and_params, ret_type) = if let Some((left, right)) = def.split_once("->return::") {
                (left.trim(), Some(right.trim()))
            } else {
                (def, None)
            };
            let (fname, params) = if let Some(lparen) = name_and_params.find('(') {
                let rparen = name_and_params.find(')').unwrap_or(name_and_params.len());
                let fname = name_and_params[..lparen].trim();
                let params_str = &name_and_params[lparen+1..rparen];
                (fname, params_str)
            } else {
                (name_and_params, "")
            };
            let param_list: Vec<(String, String)> = params.split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| {
                    let mut p = s.trim().split_whitespace();
                    (p.next().unwrap_or("").to_string(), p.next().unwrap_or("").to_string())
                }).collect();
            let ret_type = ret_type.map(|s| s.to_string());
            function_sigs.insert(fname.to_string(), (param_list.clone(), ret_type.clone()));
            current_function = Some(fname.to_string());
            functions.insert(fname.to_string(), Vec::new());
            continue;
        }
        // end(返回值) 或 end() 处理
        if let Some(ref func_name) = current_function {
            if line.trim_start().starts_with("end(") {
                let ret_val = line.trim_start()[4..].trim_end_matches(')').trim();
                function_returns.insert(func_name.clone(), ret_val.to_string());
                current_function = None;
            } else if line.trim_start() == "end" || line.trim_start() == "end()" {
                function_returns.insert(func_name.clone(), String::new());
                current_function = None;
            } else {
                functions.get_mut(func_name).unwrap().push(raw_line);
            }
            continue;
        }
        // 进入start区块
        if line == "start:" {
            in_start_section = true;
            continue;
        }
        // 收集start区块命令
        if in_start_section {
            if line == "end" {
                in_start_section = false;
            } else {
                start_commands.push(raw_line);
            }
            continue;
        }
    }

    // 递归执行代码块，基于缩进
    fn eval_block(
        block: &[String],
        functions: &HashMap<String, Vec<String>>,
        variables: &mut HashMap<String, String>,
        constants: &mut HashMap<String, String>,
        lists: &mut HashMap<String, Vec<String>>,
        var_types: &mut HashMap<String, VarType>,
        const_types: &mut HashMap<String, VarType>,
        function_sigs: &HashMap<String, (Vec<(String, String)>, Option<String>)>,
        function_returns: &HashMap<String, String>,
        parent_indent: usize,
    ) {
        use std::collections::HashSet;
        let mut called_once: HashSet<String> = HashSet::new();
        let mut i = 0;
        while i < block.len() {
            let line = &block[i];
            let cmd = line.trim();
            if cmd.is_empty() { i += 1; continue; }
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            // 块外，直接返回
            if indent < parent_indent { return; }
            let mut jumped = false;
            // 变量声明
            if cmd.starts_with("var ") {
                let rest = &cmd[4..];
                if let Some((type_and_name, value)) = rest.split_once('=') {
                    let mut parts = type_and_name.trim().split_whitespace();
                    let typ = parts.next().unwrap_or("");
                    let name = parts.next().unwrap_or("");
                    let vtype = parse_type(typ);
                    if vtype == VarType::Unknown || name.is_empty() {
                        eprintln!("变量声明语法错误: {}", cmd);
                        i += 1; continue;
                    }
                    let mut val = value.trim().to_string();
                    // 支持 input("xxx") 或 input("xxx").to_str()
                    if vtype == VarType::Str && (val.starts_with("input(") && val.ends_with(")") || val.starts_with("input(") && val.ends_with(").to_str()")) {
                        let prompt = if let Some(start) = val.find('"') {
                            let end = val.rfind('"').unwrap_or(val.len()-1);
                            &val[start+1..end]
                        } else {
                            ""
                        };
                        use std::io::{self, Write};
                        print!("{}", prompt);
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let input = input.trim_end_matches(['\r','\n']);
                        // 存储时加双引号
                        let input_val = format!("\"{}\"", input);
                        variables.insert(name.to_string(), input_val);
                        var_types.insert(name.to_string(), vtype.clone());
                        i += 1;
                        continue;
                    }
                    // 类型检查
                    let ok = match vtype {
                        VarType::Int => val.parse::<i64>().is_ok(),
                        VarType::Double => val.parse::<f64>().is_ok(),
                        VarType::Str => val.starts_with('"') && val.ends_with('"'),
                        VarType::Bool => val == "true" || val == "false",
                        _ => false,
                    };
                    if !ok {
                        eprintln!("变量 {} 类型不匹配: {}", name, val);
                    } else {
                        variables.insert(name.to_string(), val.to_string());
                    }
                } else {
                    eprintln!("变量声明语法错误: {}", cmd);
                }
            // 常量声明
            } else if cmd.starts_with("const ") {
                let rest = &cmd[6..];
                if let Some((type_and_name, value)) = rest.split_once('=') {
                    let mut parts = type_and_name.trim().split_whitespace();
                    let typ = parts.next().unwrap_or("");
                    let name = parts.next().unwrap_or("");
                    let vtype = parse_type(typ);
                    if vtype == VarType::Unknown || name.is_empty() {
                        eprintln!("常量声明语法错误: {}", cmd);
                        i += 1; continue;
                    }
                    const_types.insert(name.to_string(), vtype.clone());
                    let val = value.trim();
                    let ok = match vtype {
                        VarType::Int => val.parse::<i64>().is_ok(),
                        VarType::Double => val.parse::<f64>().is_ok(),
                        VarType::Str => val.starts_with('"') && val.ends_with('"'),
                        VarType::Bool => val == "true" || val == "false",
                        _ => false,
                    };
                    if !ok {
                        eprintln!("常量 {} 类型不匹配: {}", name, val);
                    } else {
                        constants.insert(name.to_string(), val.to_string());
                    }
                } else {
                    eprintln!("常量声明语法错误: {}", cmd);
                }
            // 支持 list 声明带类型，如 list int a = [1, 2, 3]
            } else if cmd.starts_with("list ") {
                let rest = &cmd[5..];
                let (type_and_name, value) = if let Some((type_and_name, value)) = rest.split_once('=') {
                    (type_and_name.trim(), value.trim())
                } else {
                    (rest.trim(), "")
                };
                let mut parts = type_and_name.split_whitespace();
                let typ = parts.next().unwrap_or("");
                let name = parts.next().unwrap_or(typ); // 兼容无类型写法
                let name = name.trim();
                if value.starts_with('[') && value.ends_with(']') {
                    let inner = &value[1..value.len()-1];
                    let items: Vec<String> = inner.split(',').map(|s| s.trim().to_string()).collect();
                    lists.insert(name.to_string(), items);
                }
            } else if cmd.starts_with("say ") {
                let quote_str = &cmd[4..];
                if let Some(content) = quote_str.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    println!("{}", content);
                } else {
                    let var_name = quote_str.trim();
                    // 禁止 say 后直接写函数调用或未加引号的字符串
                    if var_name.contains('(') || var_name.contains(')') || var_name.contains('"') {
                        eprintln!("say 语法错误: 只能 say 变量或 say \"字符串\"");
                    } else if let Some(val) = variables.get(var_name) {
                        // 如果变量是字符串类型，去除引号输出
                        if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                            println!("{}", &val[1..val.len()-1]);
                        } else {
                            println!("{}", val);
                        }
                    } else if let Some(val) = constants.get(var_name) {
                        if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                            println!("{}", &val[1..val.len()-1]);
                        } else {
                            println!("{}", val);
                        }
                    } else {
                        eprintln!("say 语法错误: {} 不是已定义变量或字符串", var_name);
                    }
                }
            } else if (cmd.contains("+=") || cmd.contains("-=") || cmd.contains("*=") || cmd.contains("/=") || cmd.contains("%=")) && !cmd.starts_with("const ") && !cmd.starts_with("list ") && !cmd.starts_with("call ") {
                let op = if cmd.contains("+=") { "+=" } else if cmd.contains("-=") { "-=" } else if cmd.contains("*=") { "*=" } else if cmd.contains("/=") { "/=" } else { "%=" };
                if let Some((name, value)) = cmd.split_once(op) {
                    let name = name.trim();
                    let value = value.trim();
                    if let Some(old_val) = variables.get(name) {
                        let left = old_val.parse::<f64>().unwrap_or(0.0);
                        let right = eval_math_expr(value, &variables, &constants);
                        let result = match op {
                            "+=" => left + right,
                            "-=" => left - right,
                            "*=" => left * right,
                            "/=" => left / right,
                            "%=" => left % right,
                            _ => left,
                        };
                        variables.insert(name.to_string(), result.to_string());
                    } else {
                        eprintln!("变量 {} 未定义，不能直接赋值（请用 var {} = ...）", name, name);
                    }
                }
            } else if cmd.contains('=') && !cmd.starts_with("const ") && !cmd.starts_with("list ") && !cmd.starts_with("call ") && !cmd.starts_with("if ") && !cmd.starts_with("while ") && !cmd.starts_with("for ") && !cmd.starts_with("do ") && !cmd.starts_with("switch ") {
                if let Some((name, value)) = cmd.split_once('=') {
                    let name = name.trim();
                    let value = value.trim();
                    if variables.contains_key(name) {
                        let result = eval_math_expr(value, &variables, &constants);
                        variables.insert(name.to_string(), result.to_string());
                    } else {
                        eprintln!("变量 {} 未定义，不能直接赋值（请用 var {} = ...）", name, name);
                    }
                }
            } else if cmd.starts_with("call ") {
                let call_expr = &cmd[5..].trim();
                let (fname, args): (&str, &str) = if let Some(lparen) = call_expr.find('(') {
                    let rparen = call_expr.find(')').unwrap_or(call_expr.len());
                    let fname = &call_expr[..lparen].trim();
                    let args_str = &call_expr[lparen+1..rparen];
                    (fname, args_str)
                } else {
                    (call_expr, "")
                };
                if parent_indent > 0 {
                    // 静默跳过嵌套块内的 call，不输出任何提示
                    i += 1;
                    continue;
                }
                // 主块call只执行一次
                if !called_once.insert(fname.to_string()) {
                    i += 1;
                    continue;
                }
                let arg_vals: Vec<String> = args.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                let mut local_vars = variables.clone();
                if let Some((params, _ret_type)) = function_sigs.get(fname) {
                    for ((ptype, pname), val) in params.iter().zip(arg_vals.iter()) {
                        local_vars.insert(pname.clone(), val.clone());
                        var_types.insert(pname.clone(), parse_type(ptype));
                    }
                }
                if let Some(statements) = functions.get(fname) {
                    let mut local_function_returns = function_returns.clone();
                    eval_block(statements, functions, &mut local_vars, constants, lists, var_types, const_types, function_sigs, &mut local_function_returns, 0);
                } else {
                    eprintln!("函数 {} 未定义", fname);
                }
                jumped = true;
            } else if cmd.starts_with("if ") && cmd.ends_with(":") {
                let cond = cmd[3..cmd.len()-1].trim();
                let mut inner_block = Vec::new();
                let mut j = i + 1;
                let this_indent = indent;
                while j < block.len() {
                    let l = &block[j];
                    let l_indent = l.chars().take_while(|c| c.is_whitespace()).count();
                    if l_indent <= this_indent { break; }
                    inner_block.push(block[j].clone());
                    j += 1;
                }
                let cond_result = eval_condition(cond, variables, constants);
                if cond_result {
                    eval_block(&inner_block, functions, variables, constants, lists, var_types, const_types, function_sigs, function_returns, this_indent+1);
                }
                i = j;
                jumped = true;
            } else if cmd.starts_with("while ") && cmd.ends_with(":") {
                let cond = cmd[6..cmd.len()-1].trim();
                let mut inner_block = Vec::new();
                let mut j = i + 1;
                let this_indent = indent;
                while j < block.len() {
                    let l = &block[j];
                    let l_indent = l.chars().take_while(|c| c.is_whitespace()).count();
                    if l_indent <= this_indent { break; }
                    inner_block.push(block[j].clone());
                    j += 1;
                }
                while eval_condition(cond, variables, constants) {
                    eval_block(&inner_block, functions, variables, constants, lists, var_types, const_types, function_sigs, function_returns, this_indent+1);
                }
                i = j;
                jumped = true;
            } else if cmd.starts_with("for ") && cmd.ends_with(":") {
                let cond = cmd[4..cmd.len()-1].trim();
                if let Some((var, rest)) = cond.split_once(" in ") {
                    let var = var.trim();
                    let list_name = rest.trim();
                    let items_opt = lists.get(list_name).cloned();
                    if let Some(items) = items_opt {
                        let mut inner_block = Vec::new();
                        let mut j = i + 1;
                        let this_indent = indent;
                        while j < block.len() {
                            let l = &block[j];
                            let l_indent = l.chars().take_while(|c| c.is_whitespace()).count();
                            if l_indent <= this_indent { break; }
                            inner_block.push(block[j].clone());
                            j += 1;
                        }
                        for item in items {
                            variables.insert(var.to_string(), item.clone());
                            eval_block(&inner_block, functions, variables, constants, lists, var_types, const_types, function_sigs, function_returns, this_indent+1);
                        }
                        i = j;
                        jumped = true;
                    }
                }
            } else if cmd.starts_with("do ") && cmd.ends_with(":") {
                let cond = cmd[3..cmd.len()-1].trim();
                let mut inner_block = Vec::new();
                let mut j = i + 1;
                let this_indent = indent;
                while j < block.len() {
                    let l = &block[j];
                    let l_indent = l.chars().take_while(|c| c.is_whitespace()).count();
                    if l_indent <= this_indent { break; }
                    inner_block.push(block[j].clone());
                    j += 1;
                }
                loop {
                    eval_block(&inner_block, functions, variables, constants, lists, var_types, const_types, function_sigs, function_returns, this_indent+1);
                    if !eval_condition(cond, variables, constants) { break; }
                }
                i = j;
                jumped = true;
            } else if cmd.starts_with("switch ") && cmd.ends_with(":") {
                let var = cmd[7..cmd.len()-1].trim();
                let mut j = i + 1;
                let this_indent = indent;
                let mut cases: Vec<(String, Vec<String>)> = Vec::new();
                let mut default_block = Vec::new();
                let mut current_case: Option<String> = None;
                let mut current_block: Vec<String> = Vec::new();
                while j < block.len() {
                    let l = &block[j];
                    let ltrim = l.trim();
                    let l_indent = l.chars().take_while(|c| c.is_whitespace()).count();
                    if l_indent <= this_indent { break; }
                    if ltrim.starts_with("case ") && ltrim.ends_with(":") {
                        if let Some(case_val) = current_case.take() {
                            cases.push((case_val, current_block.clone()));
                            current_block.clear();
                        }
                        current_case = Some(ltrim[5..ltrim.len()-1].trim().to_string());
                    } else if ltrim == "default:" {
                        if let Some(case_val) = current_case.take() {
                            cases.push((case_val, current_block.clone()));
                            current_block.clear();
                        }
                        current_case = None;
                    } else {
                        current_block.push(l.clone());
                    }
                    j += 1;
                }
                if let Some(case_val) = current_case.take() {
                    cases.push((case_val, current_block.clone()));
                } else if !current_block.is_empty() {
                    default_block = current_block;
                }
                let var_val = variables.get(var).or_else(|| constants.get(var)).cloned();
                let mut matched = false;
                if let Some(val) = var_val {
                    for (case_val, block2) in &cases {
                        if val == *case_val {
                            eval_block(block2, functions, variables, constants, lists, var_types, const_types, function_sigs, function_returns, this_indent+1);
                            matched = true;
                            break;
                        }
                    }
                }
                if !matched && !default_block.is_empty() {
                    eval_block(&default_block, functions, variables, constants, lists, var_types, const_types, function_sigs, function_returns, this_indent+1);
                }
                i = j;
                jumped = true;
            }
            // 变量声明类型检查增加 call 返回值类型支持
            if cmd.starts_with("var ") && cmd.contains("= call ") {
                let rest = &cmd[4..];
                if let Some((type_and_name, value)) = rest.split_once('=') {
                    let mut parts = type_and_name.trim().split_whitespace();
                    let typ = parts.next().unwrap_or("");
                    let name = parts.next().unwrap_or("");
                    let vtype = parse_type(typ);
                    if vtype == VarType::Unknown || name.is_empty() {
                        eprintln!("变量声明语法错误: {}", cmd);
                        i += 1; continue;
                    }
                    let value = value.trim();
                    if value.starts_with("call ") {
                        let call_expr = &value[5..].trim();
                        let (fname, args): (&str, &str) = if let Some(lparen) = call_expr.find('(') {
                            let rparen = call_expr.find(')').unwrap_or(call_expr.len());
                            let fname = &call_expr[..lparen].trim();
                            let args_str = &call_expr[lparen+1..rparen];
                            (fname, args_str)
                        } else {
                            (call_expr, "")
                        };
                        let arg_vals: Vec<String> = args.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        let mut local_vars = variables.clone();
                        if let Some((params, _ret_type)) = function_sigs.get(fname) {
                            for ((ptype, pname), val) in params.iter().zip(arg_vals.iter()) {
                                local_vars.insert(pname.clone(), val.clone());
                                var_types.insert(pname.clone(), parse_type(ptype));
                            }
                        }
                        if let Some(statements) = functions.get(fname) {
                            let mut local_function_returns = function_returns.clone();
                            eval_block(statements, functions, &mut local_vars, constants, lists, var_types, const_types, function_sigs, &mut local_function_returns, 0);
                            if let Some(ret) = local_function_returns.get(fname) {
                                // 类型检查
                                let mut ret_val = ret.clone();
                                let ok = match vtype {
                                    VarType::Int => ret_val.parse::<i64>().is_ok(),
                                    VarType::Double => ret_val.parse::<f64>().is_ok(),
                                    VarType::Str => {
                                        if !ret_val.starts_with('"') || !ret_val.ends_with('"') {
                                            ret_val = format!("\"{}\"", ret_val);
                                        }
                                        ret_val.starts_with('"') && ret_val.ends_with('"')
                                    },
                                    VarType::Bool => ret_val == "true" || ret_val == "false",
                                    _ => false,
                                };
                                if !ok {
                                    eprintln!("变量 {} 类型不匹配: {}", name, ret);
                                } else {
                                    variables.insert(name.to_string(), ret_val.clone());
                                    var_types.insert(name.to_string(), vtype.clone());
                                }
                            }
                        } else {
                            eprintln!("函数 {} 未定义", fname);
                        }
                    } else {
                        eprintln!("变量声明语法错误: {}", cmd);
                    }
                } else {
                    eprintln!("变量声明语法错误: {}", cmd);
                }
                i += 1;
                continue;
            }
            // 普通变量声明（排除 call 情况）
            if cmd.starts_with("var ") && !cmd.contains("= call ") {
                let rest = &cmd[4..];
                if let Some((type_and_name, value)) = rest.split_once('=') {
                    let mut parts = type_and_name.trim().split_whitespace();
                    let typ = parts.next().unwrap_or("");
                    let name = parts.next().unwrap_or("");
                    let vtype = parse_type(typ);
                    if vtype == VarType::Unknown || name.is_empty() {
                        eprintln!("变量声明语法错误: {}", cmd);
                        i += 1; continue;
                    }
                    let mut val = value.trim().to_string();
                    // 支持 input("xxx") 或 input("xxx").to_str()
                    if vtype == VarType::Str && (val.starts_with("input(") && val.ends_with(")") || val.starts_with("input(") && val.ends_with(").to_str()")) {
                        let prompt = if let Some(start) = val.find('"') {
                            let end = val.rfind('"').unwrap_or(val.len()-1);
                            &val[start+1..end]
                        } else {
                            ""
                        };
                        use std::io::{self, Write};
                        print!("{}", prompt);
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let input = input.trim_end_matches(['\r','\n']);
                        // 存储时加双引号
                        let input_val = format!("\"{}\"", input);
                        variables.insert(name.to_string(), input_val);
                        var_types.insert(name.to_string(), vtype.clone());
                        i += 1;
                        continue;
                    }
                    // 类型检查
                    let ok = match vtype {
                        VarType::Int => val.parse::<i64>().is_ok(),
                        VarType::Double => val.parse::<f64>().is_ok(),
                        VarType::Str => val.starts_with('"') && val.ends_with('"'),
                        VarType::Bool => val == "true" || val == "false",
                        _ => false,
                    };
                    if !ok {
                        eprintln!("变量 {} 类型不匹配: {}", name, val);
                    } else {
                        variables.insert(name.to_string(), val.to_string());
                    }
                } else {
                    eprintln!("变量声明语法错误: {}", cmd);
                }
            }
            // 检查是否为直接写了自定义函数名而未用 call 调用（需排除块结构、声明、say等所有已知语法）
            else if functions.contains_key(cmd)
                && !cmd.starts_with("call ")
                && !cmd.starts_with("if ")
                && !cmd.starts_with("while ")
                && !cmd.starts_with("for ")
                && !cmd.starts_with("do ")
                && !cmd.starts_with("switch ")
                && !cmd.starts_with("var ")
                && !cmd.starts_with("const ")
                && !cmd.starts_with("list ")
                && !cmd.starts_with("say ")
                && !cmd.contains("=")
            {
                eprintln!("请使用 call 语法调用函数: call {}", cmd);
            }
            // 缩进减少即块结束
            if !jumped {
                i += 1;
            }
        }
    }

    fn eval_math_expr(expr: &str, variables: &HashMap<String, String>, constants: &HashMap<String, String>) -> f64 {
        let expr = expr.trim();
        let ops = ["+", "-", "*", "/", "%"];
        for op in &ops {
            if let Some((left, right)) = expr.split_once(op) {
                let l = left.trim();
                let r = right.trim();
                let lval = variables.get(l).or_else(|| constants.get(l)).map(|s| s.as_str()).unwrap_or(l);
                let rval = variables.get(r).or_else(|| constants.get(r)).map(|s| s.as_str()).unwrap_or(r);
                let lnum = lval.parse::<f64>().unwrap_or(0.0);
                let rnum = rval.parse::<f64>().unwrap_or(0.0);
                return match *op {
                    "+" => lnum + rnum,
                    "-" => lnum - rnum,
                    "*" => lnum * rnum,
                    "/" => lnum / rnum,
                    "%" => lnum % rnum,
                    _ => 0.0,
                };
            }
        }
        let val = variables.get(expr).or_else(|| constants.get(expr)).map(|s| s.as_str()).unwrap_or(expr);
        val.parse::<f64>().unwrap_or(0.0)
    }

    fn eval_condition(cond: &str, variables: &HashMap<String, String>, constants: &HashMap<String, String>) -> bool {
        let cond = cond.replace(">=", ">=").replace("<=", "<=").replace("==", "==").replace("!=", "!=");
        let ops = [">=", "<=", "==", "!=", ">", "<"];
        for op in &ops {
            if let Some((left, right)) = cond.split_once(op) {
                let l = left.trim();
                let r = right.trim();
                let lval = variables.get(l).or_else(|| constants.get(l)).map(|s| s.as_str()).unwrap_or(l);
                let rval = variables.get(r).or_else(|| constants.get(r)).map(|s| s.as_str()).unwrap_or(r);
                match *op {
                    ">=" => return lval >= rval,
                    "<=" => return lval <= rval,
                    "==" => return lval == rval,
                    "!=" => return lval != rval,
                    ">" => return lval > rval,
                    "<" => return lval < rval,
                    _ => {}
                }
            }
        }
        false
    }

    eval_block(&start_commands, &functions, &mut variables, &mut constants, &mut lists, &mut var_types, &mut const_types, &function_sigs, &function_returns, 0);
}