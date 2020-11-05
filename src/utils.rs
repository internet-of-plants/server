use crate::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::{cell::Cell, fmt};

pub fn pad_left(src: &str, padding: &str, len: usize) -> String {
    if src.len() >= len {
        src.to_owned()
    } else {
        format!(
            "{}{}",
            padding.repeat((len - src.len()) / padding.len()),
            src
        )
    }
}

pub fn pad(src: &str, padding: &str, len: usize) -> String {
    if src.len() >= len {
        src.to_owned()
    } else {
        format!(
            "{}{}",
            src,
            padding.repeat((len - src.len()) / padding.len())
        )
    }
}

pub fn truncate(mut string: String, size: usize) -> String {
    if string.is_char_boundary(size) {
        string.truncate(size);
        string
    } else if size < string.len() {
        let mut index = size.saturating_sub(1);
        while !string.is_char_boundary(index) {
            index = index.saturating_sub(1);
        }
        string.truncate(index);
        string
    } else {
        string
    }
}

pub async fn run_migrations(url: &str) {
    use std::path::Path;
    use tokio::fs;

    let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
    let vec: Vec<Migration> = match sqlx::query_as("SELECT id FROM migrations")
        .fetch_all(&mut connection)
        .await
    {
        Ok(vec) => vec,
        Err(_) => Vec::default(),
    };
    let latest = vec.iter().max().map_or(0, |m| m.id);

    let mut files = Vec::new();
    let mut reader = fs::read_dir("migrations").await.unwrap();
    while let Some(entry) = reader.next_entry().await.unwrap() {
        let number = entry
            .file_name()
            .to_str()
            .unwrap()
            .replace(".sql", "")
            .parse::<u8>()
            .unwrap_or(0);
        if (number as i16) > latest {
            files.push(number);
        }
    }
    files.sort();
    connection.close().await.unwrap();

    for file in files {
        let connection = sqlx::PgConnection::connect(url).await.unwrap();
        let mut transaction = connection.begin().await.unwrap();
        info!("Running migration {}.sql", file);
        let path = Path::new("migrations").join(format!("{}.sql", file));
        let strings = fs::read_to_string(path).await.unwrap();
        for string in strings.split(';') {
            if string.trim().is_empty() {
                continue;
            }
            debug!("{}", string);
            sqlx::query(&format!("{};", string))
                .execute(&mut transaction)
                .await
                .unwrap();
        }
        if file == 1 {
            transaction.commit().await.unwrap();

            let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
            sqlx::query("INSERT INTO migrations (id) VALUES ($1)")
                .bind(file as i16)
                .execute(&mut connection)
                .await
                .unwrap();
        } else {
            sqlx::query("INSERT INTO migrations (id) VALUES ($1)")
                .bind(file as i16)
                .execute(&mut transaction)
                .await
                .unwrap();
            transaction.commit().await.unwrap();
        }
        let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
        info!(
            "Has migrations: {:?}",
            sqlx::query_as::<_, (i16,)>("SELECT id FROM migrations ORDER BY id ASC")
                .fetch_all(&mut connection)
                .await
        );
    }
}

pub fn http_log(info: warp::log::Info) {
    thread_local! {
        static PATH_LEN: Cell<usize> = Cell::new(0);
        static DURATION_LEN: Cell<usize> = Cell::new(0);
        static IP_LEN: Cell<usize> = Cell::new(0);
    }

    let duration = format!("{:.4}", utils::Duration(info.elapsed()));
    let duration_len = DURATION_LEN.with(|c| c.replace(c.get().max(duration.len())));

    let path_len = PATH_LEN.with(|c| c.replace(c.get().max(info.path().len())));

    let ip = utils::OptFmt(info.remote_addr()).to_string();
    let ip_len = IP_LEN.with(|c| c.replace(c.get().max(ip.len())));

    let method_len = "DELETE".len();
    info!(
        "{} {} {} {} {} {}",
        info.status().as_u16(),
        utils::pad(info.method().as_str(), " ", method_len),
        utils::pad(info.path(), " ", path_len),
        utils::pad_left(&duration, " ", duration_len),
        utils::pad(&ip, " ", ip_len),
        //utils::OptFmt(info.referer()),
        utils::OptFmt(info.user_agent().map(|u| utils::truncate(u.to_owned(), 30))),
    );
}

/// Wrapper to allow optional formatting of a type
pub struct OptFmt<T>(pub Option<T>);

impl<T: fmt::Display> fmt::Display for OptFmt<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref t) = self.0 {
            fmt::Display::fmt(t, f)
        } else {
            f.write_str("-")
        }
    }
}

pub struct Duration(pub std::time::Duration);

// Changes microsecond unity from 'µs' to 'mS' to avoid breaking monospace fonts
// that deal poorly with utf-8
impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Formats a floating point number in decimal notation.
        ///
        /// The number is given as the `integer_part` and a fractional part.
        /// The value of the fractional part is `fractional_part / divisor`. So
        /// `integer_part` = 3, `fractional_part` = 12 and `divisor` = 100
        /// represents the number `3.012`. Trailing zeros are omitted.
        ///
        /// `divisor` must not be above 100_000_000. It also should be a power
        /// of 10, everything else doesn't make sense. `fractional_part` has
        /// to be less than `10 * divisor`!
        fn fmt_decimal(
            f: &mut fmt::Formatter<'_>,
            mut integer_part: u64,
            mut fractional_part: u32,
            mut divisor: u32,
        ) -> fmt::Result {
            // Encode the fractional part into a temporary buffer. The buffer
            // only need to hold 9 elements, because `fractional_part` has to
            // be smaller than 10^9. The buffer is prefilled with '0' digits
            // to simplify the code below.
            let mut buf = [b'0'; 9];

            // The next digit is written at this position
            let mut pos = 0;

            // We keep writing digits into the buffer while there are non-zero
            // digits left and we haven't written enough digits yet.
            while fractional_part > 0 && pos < f.precision().unwrap_or(9) {
                // Write new digit into the buffer
                buf[pos] = b'0' + (fractional_part / divisor) as u8;

                fractional_part %= divisor;
                divisor /= 10;
                pos += 1;
            }

            // If a precision < 9 was specified, there may be some non-zero
            // digits left that weren't written into the buffer. In that case we
            // need to perform rounding to match the semantics of printing
            // normal floating point numbers. However, we only need to do work
            // when rounding up. This happens if the first digit of the
            // remaining ones is >= 5.
            if fractional_part > 0 && fractional_part >= divisor * 5 {
                // Round up the number contained in the buffer. We go through
                // the buffer backwards and keep track of the carry.
                let mut rev_pos = pos;
                let mut carry = true;
                while carry && rev_pos > 0 {
                    rev_pos -= 1;

                    // If the digit in the buffer is not '9', we just need to
                    // increment it and can stop then (since we don't have a
                    // carry anymore). Otherwise, we set it to '0' (overflow)
                    // and continue.
                    if buf[rev_pos] < b'9' {
                        buf[rev_pos] += 1;
                        carry = false;
                    } else {
                        buf[rev_pos] = b'0';
                    }
                }

                // If we still have the carry bit set, that means that we set
                // the whole buffer to '0's and need to increment the integer
                // part.
                if carry {
                    integer_part += 1;
                }
            }

            // Determine the end of the buffer: if precision is set, we just
            // use as many digits from the buffer (capped to 9). If it isn't
            // set, we only use all digits up to the last non-zero one.
            let end = f.precision().map(|p| std::cmp::min(p, 9)).unwrap_or(pos);

            // If we haven't emitted a single fractional digit and the precision
            // wasn't set to a non-zero value, we don't print the decimal point.
            if end == 0 {
                write!(f, "{}", integer_part)
            } else {
                // SAFETY: We are only writing ASCII digits into the buffer and it was
                // initialized with '0's, so it contains valid UTF8.
                let s = unsafe { std::str::from_utf8_unchecked(&buf[..end]) };

                // If the user request a precision > 9, we pad '0's at the end.
                let w = f.precision().unwrap_or(pos);
                write!(f, "{}.{:0<width$}", integer_part, s, width = w)
            }
        }

        // Print leading '+' sign if requested
        if f.sign_plus() {
            write!(f, "+")?;
        }

        if self.0.as_secs() > 0 {
            fmt_decimal(f, self.0.as_secs(), self.0.subsec_nanos(), 100_000_000)?;
            f.write_str("s")
        } else if self.0.subsec_nanos() >= 1_000_000 {
            fmt_decimal(
                f,
                self.0.subsec_nanos() as u64 / 1_000_000,
                self.0.subsec_nanos() % 1_000_000,
                100_000,
            )?;
            f.write_str("ms")
        } else if self.0.subsec_nanos() >= 1_000 {
            fmt_decimal(
                f,
                self.0.subsec_nanos() as u64 / 1_000,
                self.0.subsec_nanos() % 1_000,
                100,
            )?;
            f.write_str("mS")
        } else {
            fmt_decimal(f, self.0.subsec_nanos() as u64, 0, 1)?;
            f.write_str("ns")
        }
    }
}

pub fn random_name() -> String {
    let or_random = || random_string(30);

    let name = names::Generator::default().next().unwrap_or_else(or_random);
    let name2 = names::Generator::default().next().unwrap_or_else(or_random);
    let name3 = names::Generator::default().next().unwrap_or_else(or_random);
    format!("{}-{}-{}", name, name2, name3)
}

pub fn random_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .collect()
}

pub fn hash_password(password: &str) -> Result<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    Ok(bcrypt::verify(password, hash)?)
}

pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Serializer, Deserialize, Deserializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: Display,
              S: Serializer
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where T: FromStr,
              T::Err: Display,
              D: Deserializer<'de>
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

pub mod float {
    use serde::{Serializer, Deserialize, Deserializer};

    pub fn serialize<S>(value: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        if let Some(value) = value {
            serializer.serialize_f32(*value)
        } else {
            serializer.serialize_f32(0.)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
        where D: Deserializer<'de>
    {
        Ok(Option::<f32>::deserialize(deserializer)?.unwrap_or(0.))
    }
}
