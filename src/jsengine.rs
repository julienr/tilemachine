use std::{sync::atomic::AtomicBool, sync::atomic::Ordering};

static mut PLATFORM_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct JSEngine {
    isolate: v8::OwnedIsolate,
}

impl Default for JSEngine {
    fn default() -> Self {
        // Doing platform initialization twice seems to lead to "Invalid global state"
        // so it looks like we need a singleton to ensure this is done exactly once
        let initialized = unsafe { PLATFORM_INITIALIZED.load(Ordering::Relaxed) };
        if !initialized {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
            unsafe {
                PLATFORM_INITIALIZED.store(true, Ordering::Relaxed);
            }
        }

        let isolate = v8::Isolate::new(Default::default());
        JSEngine { isolate }
    }
}

fn run_on_pixel(
    func: &v8::Local<v8::Function>,
    r: u8,
    g: u8,
    b: u8,
    context_scope: &mut v8::ContextScope<v8::HandleScope>,
) -> [u8; 3] {
    let call_scope = &mut v8::HandleScope::new(context_scope);
    let args = [
        v8::Number::new(call_scope, r as f64).into(),
        v8::Number::new(call_scope, g as f64).into(),
        v8::Number::new(call_scope, b as f64).into(),
    ];
    let function_this: v8::Local<'_, v8::Value> = v8::null(call_scope).into();
    let return_value = func.call(call_scope, function_this, &args).unwrap();
    let return_scope = &mut v8::HandleScope::new(call_scope);
    if !return_value.is_array() {
        panic!(
            "Expected an array as return type, got {:?}",
            return_value.type_repr()
        );
    }
    let return_array = v8::Local::<v8::Array>::try_from(return_value).unwrap();
    let mut extract_channel = |i: u32| -> u8 {
        let mut v = return_array
            .get_index(return_scope, i)
            .unwrap()
            .number_value(return_scope)
            .unwrap();
        if v > 255.0 {
            v = 255.0;
        }
        v as u8
    };
    [extract_channel(0), extract_channel(1), extract_channel(2)]
}

impl JSEngine {
    pub fn compile<F>(&mut self, code: &str, callback: &mut F)
    where
        F: FnMut(&mut dyn FnMut(u8, u8, u8) -> [u8; 3]),
    {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(handle_scope);
        let context_scope = &mut v8::ContextScope::new(handle_scope, context);
        let code = v8::String::new(context_scope, code).unwrap();
        let args = [
            v8::String::new(context_scope, "r").unwrap(),
            v8::String::new(context_scope, "g").unwrap(),
            v8::String::new(context_scope, "b").unwrap(),
        ];
        let function = v8::script_compiler::compile_function(
            context_scope,
            v8::script_compiler::Source::new(code, None),
            &args,
            &[],
            v8::script_compiler::CompileOptions::NoCompileOptions,
            v8::script_compiler::NoCacheReason::NoReason,
        )
        .unwrap();

        {
            callback(&mut |r: u8, g: u8, b: u8| -> [u8; 3] {
                run_on_pixel(&function, r, g, b, context_scope)
            });
        }
    }

    pub fn compile_and_run_once(&mut self, code: &str, r: u8, g: u8, b: u8) -> [u8; 3] {
        let mut output: [u8; 3] = [0, 0, 0];
        self.compile(code, &mut |compiled_func| {
            output = compiled_func(r, g, b);
        });
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_js_engine_exec() {
        let mut engine = JSEngine::default();
        let code = "return [r * 2, g * 3, b * 4]";
        let result = engine.compile_and_run_once(code, 1, 1, 1);
        assert!(result.iter().eq([2, 3, 4].iter()));
    }
}
