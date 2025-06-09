
use rand::Rng;

use microkv::MicroKV;

fn create_records(num_records: usize) -> Vec<(String, String)> {
    let charset = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    let mut records: Vec<(String, String)> = Vec::new();

    for _ in 0..num_records {
        let key: String = (0..10)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();

        let val: String = (0..10).
            map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();

        records.push((key, val));
    }
    records
}

#[test]
fn test_kv() {
    let args: Vec<String> = std::env::args().collect();
    let num_records = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or_else(|_| {
            eprintln!("Invalid number '{}', falling back to random", args[1]);
            rand::thread_rng().gen_range(1..100)
        })
    } else {
        rand::thread_rng().gen_range(1..100)
    };

    println!("Running test with {} records", num_records);

    let kv: MicroKV = MicroKV::new("test_kv");
    let records = create_records(num_records);
    let start = std::time::Instant::now();
    for (key, val) in &records {
        kv.put(&key, &val).expect("cannot insert value");
    }

    for (key, val) in &records {
        let res: String = kv.get_unwrap(&key).expect("cannot retrieve value");
        assert_eq!(val, &res);
    }
    let elapsed = start.elapsed();
    println!("Time taken to insert and retrieve {} records: {:?}", num_records, elapsed);
}
