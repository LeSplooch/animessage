use super::*;

pub enum Variable {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Variable>),
}

impl Variable {
    /// Creates a new variable. Returns `Some(variable)` if vars_cache is `None`, else puts it in cache and returns `None`.
    // TODO: Maybe change the return type to a Result<Variable>.
    pub fn new(
        var_name: &str,
        var_type: &str,
        var_unparsed: &str,
        vars_cache: Option<&mut HashMap<String, Variable>>,
        debug: bool,
    ) -> Option<Variable> {
        if debug && vars_cache.is_some() {
            debug!("Adding variable")
        }
        let mut ret = None;
        match var_type {
            "str" => {
                let var = Variable::String(var_unparsed.to_string());
                match vars_cache {
                    None => ret = Some(var),
                    Some(vars_cache) => {
                        vars_cache.insert(var_name.to_string(), var);
                    }
                }
            }
            "int" => {
                let tmp_var: i64 = var_unparsed.parse().unwrap_or_else(|_| {
                    error!(r#"Can't create an integer from "{}""#, var_unparsed);
                    std::process::exit(0);
                });
                let var = Variable::Int(tmp_var);
                match vars_cache {
                    None => ret = Some(var),
                    Some(vars_cache) => {
                        vars_cache.insert(var_name.to_string(), var);
                    }
                }
            }
            "float" => {
                let tmp_var: f64 = var_unparsed.parse().unwrap_or_else(|_| {
                    error!(
                        r#"Can't create a floating point number from "{}""#,
                        var_unparsed
                    );
                    std::process::exit(0);
                });
                let var = Variable::Float(tmp_var);
                match vars_cache {
                    None => ret = Some(var),
                    Some(vars_cache) => {
                        vars_cache.insert(var_name.to_string(), var);
                    }
                }
            }
            "bool" => {
                let tmp_var: bool = var_unparsed.parse().unwrap_or_else(|_| {
                    error!(
                        r#"Can't create a floating point number from "{}""#,
                        var_unparsed
                    );
                    std::process::exit(0);
                });
                let var = Variable::Bool(tmp_var);
                match vars_cache {
                    None => ret = Some(var),
                    Some(vars_cache) => {
                        vars_cache.insert(var_name.to_string(), var);
                    }
                }
            }
            "list" => {
                let mut var_unparsed_split = var_unparsed.split(" ");
                let vars_type = var_unparsed_split.nth(0).unwrap_or_else(|| {
                    error!("You need to define the variable type as the first argument.");
                    std::process::exit(0)
                });

                let mut list = Vec::new();
                for v in var_unparsed_split {
                    let new_var = Variable::new(var_name, vars_type, v, None, debug).unwrap(); // Normalement on est sûrs qu'il est un Some. A re-vérifier.
                    list.push(new_var);
                }

                let var = Variable::List(list);
                match vars_cache {
                    None => ret = Some(var),
                    Some(vars_cache) => {
                        vars_cache.insert(var_name.to_string(), var);
                    }
                }
            }
            _ => {
                error!("Your variable isn't a parsable item.");
                std::process::exit(0);
            }
        }

        ret
    }
}
