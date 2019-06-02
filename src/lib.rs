//!  Extract and inject [W3C TraceContext](https://w3c.github.io/trace-context/) headers.
//!
//! ## Examples
//! ```
//! let mut headers = http::HeaderMap::new();
//! headers.insert(
//!     "traceparent",
//!     "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01".parse().unwrap()
//! );
//! 
//! let context = trace_context::TraceContext::extract(&headers).unwrap();
//! 
//! assert_eq!(context.trace_id(), u128::from_str_radix("0af7651916cd43dd8448eb211c80319c", 16).unwrap());
//! assert_eq!(context.parent_id(), u64::from_str_radix("00f067aa0ba902b7", 16).ok());
//! assert_eq!(context.sampled(), true);
//! ```

#![deny(unsafe_code)]

use rand::Rng;
use std::fmt;

/// A TraceContext object
#[derive(Debug)]
pub struct TraceContext {
    id: u64,
    version: u8,
    trace_id: u128,
    parent_id: Option<u64>,
    flags: u8,
}

impl TraceContext {
    /// Create and return TraceContext object based on `traceparent` HTTP header.
    ///
    /// ## Examples
    /// ```
    /// let mut headers = http::HeaderMap::new();
    /// headers.insert("traceparent", "00-0af7651916cd43dd8448eb211c80319c-00f067aa0ba902b7-01".parse().unwrap());
    /// 
    /// let context = trace_context::TraceContext::extract(&headers).unwrap();
    /// 
    /// assert_eq!(context.trace_id(), u128::from_str_radix("0af7651916cd43dd8448eb211c80319c", 16).unwrap());
    /// assert_eq!(context.parent_id(), u64::from_str_radix("00f067aa0ba902b7",
    /// 16).ok());
    /// assert_eq!(context.sampled(), true);
    /// ```
    pub fn extract(headers: &http::HeaderMap) -> Result<Self, std::num::ParseIntError> {
        let mut rng = rand::thread_rng();

        let traceparent = match headers.get("traceparent") {
            Some(header) => header.to_str().unwrap(),
            None => return Ok(Self::new_root()),
        };

        let parts: Vec<&str> = traceparent.split('-').collect();

        Ok(Self {
            id: rng.gen(),
            version: u8::from_str_radix(parts[0], 16)?,
            trace_id: u128::from_str_radix(parts[1], 16)?,
            parent_id: Some(u64::from_str_radix(parts[2], 16)?),
            flags: u8::from_str_radix(parts[3], 16)?
        })
    }

    pub fn new_root() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            id: rng.gen(),
            version: 0,
            trace_id: rng.gen(),
            parent_id: None,
            flags: 1
        }
    }

    /// Add the traceparent header to the http headers
    ///
    /// ## Examples
    /// ```
    /// let mut input_headers = http::HeaderMap::new();
    /// input_headers.insert("traceparent", "00-00000000000000000000000000000001-0000000000000002-01".parse().unwrap());
    ///
    /// let parent = trace_context::TraceContext::extract(&input_headers).unwrap();
    ///
    /// let mut output_headers = http::HeaderMap::new();
    /// parent.inject(&mut output_headers);
    ///
    /// let child = trace_context::TraceContext::extract(&output_headers).unwrap();
    ///
    /// assert_eq!(child.version(), parent.version());
    /// assert_eq!(child.trace_id(), parent.trace_id());
    /// assert_eq!(child.parent_id(), Some(parent.id()));
    /// assert_eq!(child.flags(), parent.flags());
    /// ```
    pub fn inject(&self, headers: &mut http::HeaderMap) {
        headers.insert("traceparent", format!("{}", self).parse().unwrap());
    }

    pub fn child(&self) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            id: rng.gen(),
            version: self.version,
            trace_id: self.trace_id,
            parent_id: Some(self.id),
            flags: self.flags,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    pub fn parent_id(&self) -> Option<u64> {
        self.parent_id
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    /// Returns true if the trace is sampled
    ///
    /// ## Examples
    /// ```
    /// let mut headers = http::HeaderMap::new();
    /// headers.insert("traceparent", "00-00000000000000000000000000000001-0000000000000002-01".parse().unwrap());
    /// let context = trace_context::TraceContext::extract(&headers).unwrap();
    /// assert_eq!(context.sampled(), true);
    /// ```
    pub fn sampled(&self) -> bool {
        (self.flags & 0b00000001) == 1
    }

    /// Change sampled flag
    ///
    /// ## Examples
    /// ```
    /// let mut context = trace_context::TraceContext::new_root();
    /// assert_eq!(context.sampled(), true);
    /// context.set_sampled(false);
    /// assert_eq!(context.sampled(), false);
    /// ```
    pub fn set_sampled(&mut self, sampled: bool) {
        let x = sampled as u8;
        self.flags ^= (x ^ self.flags) & (1 << 0);
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}-{:032}-{:016x}-{:02x}",
            self.version, self.trace_id, self.id, self.flags
        )
    }
}

#[cfg(test)]
mod test {
    mod extract {
        #[test]
        fn default() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut headers = http::HeaderMap::new();
            headers.insert("traceparent", "00-01-deadbeef-00".parse()?);
            let context = crate::TraceContext::extract(&headers)?;
            assert_eq!(context.version(), 0);
            assert_eq!(context.trace_id(), 1);
            assert_eq!(context.parent_id().unwrap(), 3735928559);
            assert_eq!(context.flags(), 0);
            assert_eq!(context.sampled(), false);
            Ok(())
        }

        #[test]
        fn no_header() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
            let headers = http::HeaderMap::new();
            let context = crate::TraceContext::extract(&headers)?;
            assert_eq!(context.version(), 0);
            assert_eq!(context.parent_id(), None);
            assert_eq!(context.flags(), 1);
            assert_eq!(context.sampled(), true);
            Ok(())
        }

        #[test]
        fn not_sampled() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut headers = http::HeaderMap::new();
            headers.insert("traceparent", "00-01-02-00".parse().unwrap());
            let context = crate::TraceContext::extract(&headers)?;
            assert_eq!(context.sampled(), false);
            Ok(())
        }

        #[test]
        fn sampled() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut headers = http::HeaderMap::new();
            headers.insert("traceparent", "00-01-02-01".parse().unwrap());
            let context = crate::TraceContext::extract(&headers)?;
            assert_eq!(context.sampled(), true);
            Ok(())
        }
    }
}
