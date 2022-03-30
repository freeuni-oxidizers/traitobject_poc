use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Display;

// all function pointers will be serialized as integer offsets relative to this
#[used]
#[no_mangle]
pub static RELATIVE_FNS_BASE: &() = &();

// helper for storing function pointers
#[derive(Debug, Serialize, Deserialize)]
struct SerFnPtr(usize);

impl SerFnPtr {
    fn new(p: usize) -> SerFnPtr {
        let base = RELATIVE_FNS_BASE as *const () as usize;
        SerFnPtr(p.wrapping_sub(base))
    }

    fn to_fp(&self) -> usize {
        let base = RELATIVE_FNS_BASE as *const () as usize;
        self.0.wrapping_add(base)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DataAndFunction<T> {
    s: T,
    f: SerFnPtr,
}

// why do we need this DeserializeOwned?
impl<T: Serialize + DeserializeOwned> DataAndFunction<T> {
    fn new(s: T, f: fn(T) -> i32) -> Self {
        DataAndFunction {
            s,
            f: SerFnPtr::new(f as usize),
        }
    }
}

// This trait can be serialized and deserialized without knowing actual concrete type
trait Helloer: serde_traitobject::Serialize + serde_traitobject::Deserialize {
    fn hello(&self);
}

// This is where call to self.f is actually done
impl<T: Serialize + DeserializeOwned + Display + Clone> Helloer for DataAndFunction<T> {
    fn hello(&self) {
        // casting SerFnPtr to fp
        let sfp = self.f.to_fp();
        let fp: fn(T) -> i32;
        unsafe {
            fp = std::mem::transmute(sfp);
        };

        println!("Hello, {}", self.s);
        println!("Hello length is {}", fp(self.s.clone()));
    }
}

// dump container
#[derive(Serialize, Deserialize)]
struct Container {
    #[serde(with = "serde_traitobject")]
    dataandfn: Box<dyn Helloer>,
}

fn main() {
    let df = DataAndFunction::new("World".to_string(), |s: String| s.len() as i32);

    let msg = Container {
        dataandfn: Box::new(df),
    };

    let serialized = serde_json::to_string(&msg).unwrap();
    println!("serialized json: \n{}", &serialized);

    let deserialized: Container = serde_json::from_str(&serialized).unwrap();
    deserialized.dataandfn.hello();
}
