# GXT Extism Plugin

**ATTENTION**: The gxt library needs a source of randomness to work and therefor has an indirect dependency on `getrandom`.
Sadly, its not possible to compile `getrandom` in a way that works out of the box, so for the time being its
required to provide a host function thats responsible for creating the required random numbers.

Below is the function I wrote and tested, so it should work:
```rs
extism::host_fn!(get_random_bytes(_user_data: (); len: u64) -> Vec<u8> {
    let range = Uniform::new(u8::MIN, u8::MAX).unwrap();
    let values: Vec<u8> = rand::rng().sample_iter(&range).take(len as usize).collect();
    Ok(values)
});
```
