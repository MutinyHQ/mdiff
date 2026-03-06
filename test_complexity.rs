// Test file to demonstrate complexity analysis
use std::collections::HashMap;

pub fn simple_function() {
    println!("Hello world");
}

pub fn complex_function(data: Vec<String>) -> Result<HashMap<String, i32>, Box<dyn std::error::Error>> {
    let mut result = HashMap::new();
    
    for item in data {
        if item.len() > 10 {
            match item.parse::<i32>() {
                Ok(num) => {
                    if num > 0 {
                        for i in 0..num {
                            if i % 2 == 0 {
                                result.insert(format!("key_{}", i), i * 2);
                            }
                        }
                    }
                }
                Err(_) => {
                    // This is unsafe and has unwrap calls
                    unsafe {
                        let ptr = item.as_ptr();
                        let value = ptr.read();
                        result.insert(item.clone(), value as i32);
                    }
                    let fallback = item.parse::<i32>().unwrap();
                    result.insert("fallback".to_string(), fallback);
                }
            }
        } else {
            result.insert(item.clone(), item.len() as i32);
        }
    }
    
    Ok(result)
}