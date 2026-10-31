use rsomics_common::{Result, RsomicsError};

const MULTIPLIER: u128 = (2549297995355413924u128 << 64) | 4865540595714422341;

struct Pcg64 {
    state: u128,
    increment: u128,
    cached: Option<u32>,
}

impl Pcg64 {
    fn from_seed(seed: u128) -> Self {
        let words = seed_words(seed);
        let initial_state = (u128::from(words[0]) << 64) | u128::from(words[1]);
        let sequence = (u128::from(words[2]) << 64) | u128::from(words[3]);
        let mut generator = Self {
            state: 0,
            increment: (sequence << 1) | 1,
            cached: None,
        };
        generator.step();
        generator.state = generator.state.wrapping_add(initial_state);
        generator.step();
        generator
    }

    fn step(&mut self) {
        self.state = self
            .state
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(self.increment);
    }

    fn next_u64(&mut self) -> u64 {
        self.step();
        let value = (self.state >> 64) as u64 ^ self.state as u64;
        value.rotate_right((self.state >> 122) as u32)
    }

    fn next_u32(&mut self) -> u32 {
        if let Some(value) = self.cached.take() {
            return value;
        }
        let value = self.next_u64();
        self.cached = Some((value >> 32) as u32);
        value as u32
    }

    fn bounded(&mut self, upper: u64) -> u64 {
        match upper {
            0 => 0,
            0xffff_ffff => u64::from(self.next_u32()),
            1..=0xffff_fffe => u64::from(self.bounded_u32(upper as u32)),
            u64::MAX => self.next_u64(),
            _ => self.bounded_u64(upper),
        }
    }

    fn bounded_u32(&mut self, upper: u32) -> u32 {
        let range = u64::from(upper) + 1;
        let mut product = u64::from(self.next_u32()) * range;
        let mut low = u64::from(product as u32);
        if low < range {
            let threshold = u64::from(u32::MAX - upper) % range;
            while low < threshold {
                product = u64::from(self.next_u32()) * range;
                low = u64::from(product as u32);
            }
        }
        (product >> 32) as u32
    }

    fn bounded_u64(&mut self, upper: u64) -> u64 {
        let range = u128::from(upper) + 1;
        let mut product = u128::from(self.next_u64()) * range;
        let mut low = product as u64;
        if u128::from(low) < range {
            let threshold = (u64::MAX - upper) % (upper + 1);
            while low < threshold {
                product = u128::from(self.next_u64()) * range;
                low = product as u64;
            }
        }
        (product >> 64) as u64
    }
}

pub(crate) fn choose(population: u64, depth: u64, seed: u128) -> Result<Vec<u64>> {
    let mut generator = Pcg64::from_seed(seed);
    if population > 10000 && depth > population / 20 {
        tail_shuffle(&mut generator, population, depth)
    } else {
        floyd(&mut generator, population, depth)
    }
}

fn floyd(generator: &mut Pcg64, population: u64, depth: u64) -> Result<Vec<u64>> {
    let mut selected = workspace(depth, "selected reads")?;
    let mask = bit_mask((1.2 * depth as f64) as u64);
    let mut table = workspace(mask + 1, "Floyd lookup table")?;
    table.resize((mask + 1) as usize, u64::MAX);
    for index in population - depth..population {
        let value = generator.bounded(index);
        let mut slot = (value & mask) as usize;
        while table[slot] != u64::MAX && table[slot] != value {
            slot = (slot + 1) & mask as usize;
        }
        if table[slot] == u64::MAX {
            table[slot] = value;
            selected.push(value);
        } else {
            let mut slot = (index & mask) as usize;
            while table[slot] != u64::MAX {
                slot = (slot + 1) & mask as usize;
            }
            table[slot] = index;
            selected.push(index);
        }
    }
    Ok(selected)
}

fn tail_shuffle(generator: &mut Pcg64, population: u64, depth: u64) -> Result<Vec<u64>> {
    let mut selected = workspace(population, "tail-shuffle population")?;
    selected.extend(0..population);
    for index in ((population - depth).max(1)..population).rev() {
        let other = generator.bounded(index) as usize;
        selected.swap(index as usize, other);
    }
    selected.copy_within((population - depth) as usize.., 0);
    selected.truncate(depth as usize);
    Ok(selected)
}

fn workspace(length: u64, role: &str) -> Result<Vec<u64>> {
    let length = usize::try_from(length)
        .map_err(|_| RsomicsError::ConfigError(format!("{role} exceeds addressable memory")))?;
    let mut values = Vec::new();
    values.try_reserve_exact(length).map_err(|error| {
        RsomicsError::ConfigError(format!("allocating {role} ({length} entries): {error}"))
    })?;
    Ok(values)
}

fn bit_mask(mut value: u64) -> u64 {
    for shift in [1, 2, 4, 8, 16, 32] {
        value |= value >> shift;
    }
    value
}

fn seed_words(seed: u128) -> [u64; 4] {
    let mut pool = [0u32; 4];
    let mut hash_constant = 0x43b0_d7e5;
    for (index, value) in pool.iter_mut().enumerate() {
        *value = hash_mix((seed >> (32 * index)) as u32, &mut hash_constant);
    }
    for source in 0..pool.len() {
        for destination in 0..pool.len() {
            if source != destination {
                let hash = hash_mix(pool[source], &mut hash_constant);
                let mut value = 0xca01_f9ddu32
                    .wrapping_mul(pool[destination])
                    .wrapping_sub(0x4973_f715u32.wrapping_mul(hash));
                value ^= value >> 16;
                pool[destination] = value;
            }
        }
    }
    let mut state = [0u32; 8];
    let mut hash_constant = 0x8b51_f9dd;
    for (index, value) in state.iter_mut().enumerate() {
        let mut data = pool[index % pool.len()] ^ hash_constant;
        hash_constant = hash_constant.wrapping_mul(0x58f3_8ded);
        data = data.wrapping_mul(hash_constant);
        *value = data ^ (data >> 16);
    }
    std::array::from_fn(|index| {
        u64::from(state[2 * index]) | (u64::from(state[2 * index + 1]) << 32)
    })
}

fn hash_mix(mut value: u32, hash_constant: &mut u32) -> u32 {
    value ^= *hash_constant;
    *hash_constant = hash_constant.wrapping_mul(0x931e_8875);
    value = value.wrapping_mul(*hash_constant);
    value ^ (value >> 16)
}
