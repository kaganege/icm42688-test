pub(crate) unsafe fn put_str_raw(value: &str) {
  for char in value.chars() {
    crate::putchar_raw(char as _);
  }
}

::custom_print::define_macros!(#[macro_export] { print, println, dbg }, concat, |value: &str| {
  unsafe {
    crate::std::print::put_str_raw(value);
  }
});

::custom_print::define_macros!(#[macro_export] { eprint, eprintln }, concat, |value: &str| {
  unsafe {
    crate::std::print::put_str_raw(format!("error: {value}").as_str());
  }
});

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
  (min $( $args:expr ),*) => {
    print!( $( $args ),* );
  };

  ($( $args:expr ),*) => {
    print!("[DEBUG] ");
    print!( $( $args ),* );
  };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debugln {
  (min $( $args:expr ),*) => {
    println!( $( $args ),* );
  };

  ($( $args:expr ),*) => {
    print!("[DEBUG] ");
    println!( $( $args ),* );
  };
}
