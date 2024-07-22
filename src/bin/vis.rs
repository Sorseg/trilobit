fn main() {
    #[cfg(not(feature="vis"))] {
        println!("Run this example with --features vis to see visualization");
        std::process::exit(1);
    }
    #[cfg(feature="vis")] {
        trilobit::vis::vis();
    }

}