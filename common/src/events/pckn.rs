#![deny(missing_docs)]

use Match::*;

/// A Port, Channel, Key, NoteId (PCKN) tuple.
///
/// CLAP addresses notes and voices using this 4-value tuple: `port`, `channel`, `key` and `note_id`.
/// Each of the components in this PCKN can either be a specific value, or a wildcard that matches
/// any value in that part of the tuple. This is representing using the [`Match`] enum.
///
/// For instance, a [`Pckn`] of `(0, 3, All, All)` will match all voices
/// on channel 3 of port 0. And a [`Pckn`] of `(All, 0, 60, All)` will match
/// all channel 0 key 60 voices, independent of port or note id.
///
/// See the [`matches`](Pckn::matches) for an implementation of the PCKN matching logic that you
/// can use to match incoming events against active voices.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Pckn {
    /// The Note Port the plugin received this event on. See the Note Ports extension.
    pub port: Match<u16>,
    /// The Channel the note is on, akin to MIDI1 channels. This is usually in the `0..=15` range.
    pub channel: Match<u16>,
    /// The note's Key. This is the same representation as MIDI1 Key numbers,
    /// `60` being a Middle C, and is in the `0..=127` range.
    pub key: Match<u16>,
    /// The unique ID of this note. This is used to distinguish between multiple overlapping
    /// notes that play the same key. This is in the `0..i32::MAX` range.
    pub note_id: Match<u32>,
}

impl Pckn {
    /// Constructs a new PCKN tuple from each of its components.
    pub fn new(
        port: impl Into<Match<u16>>,
        channel: impl Into<Match<u16>>,
        key: impl Into<Match<u16>>,
        note_id: impl Into<Match<u32>>,
    ) -> Self {
        Self {
            port: port.into(),
            channel: channel.into(),
            key: key.into(),
            note_id: note_id.into(),
        }
    }

    /// Returns a [`Pckn`] tuple that matches *all* events, i.e. all of its components are set to
    /// [`Match::All`].
    #[inline]
    pub const fn match_all() -> Self {
        Self {
            port: All,
            channel: All,
            key: All,
            note_id: All,
        }
    }

    /// Returns `true` if this PCKN tuple matches the given one, considering both specific values
    /// and wildcard [`Match::All`] values.
    ///
    /// # Examples
    ///
    /// ```
    /// use clack_common::events::{Match, Pckn};
    ///
    /// assert!(Pckn::new(0u16, 0u16, 60u16, 42u32).matches(&Pckn::new(0u16, 0u16, 60u16, Match::All)));
    /// ```
    pub fn matches(&self, other: &Pckn) -> bool {
        if !self.port.matches(&other.port) {
            return false;
        }
        if !self.channel.matches(&other.channel) {
            return false;
        }
        if !self.key.matches(&other.key) {
            return false;
        }
        self.note_id.matches(&other.note_id)
    }

    // Raw accessors

    /// Constructs a new PCKN tuple from its raw, C-FFI compatible components.
    ///
    /// Components set to any negative value (usually `-1`) are interpreted as [`Match::All`], while
    /// any other value is interpreted as [`Match::Specific`].
    #[inline]
    pub const fn from_raw(port: i16, channel: i16, key: i16, note_id: i32) -> Self {
        Self {
            port: Match::<u16>::from_raw(port),
            channel: Match::<u16>::from_raw(channel),
            key: Match::<u16>::from_raw(key),
            note_id: Match::<u32>::from_raw(note_id),
        }
    }

    /// Returns the raw, C-FFI compatible Port component of this PCKN.
    ///
    /// This returns `-1` if the port is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_port(&self) -> i16 {
        match self.port {
            Specific(p) => p as i16,
            All => -1,
        }
    }

    /// Returns the raw, C-FFI compatible Channel component of this PCKN.
    ///
    /// This returns `-1` if the Channel is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_channel(&self) -> i16 {
        match self.channel {
            Specific(p) => p as i16,
            All => -1,
        }
    }

    /// Returns the raw, C-FFI compatible Key component of this PCKN.
    ///
    /// This returns `-1` if the Key is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_key(&self) -> i16 {
        match self.key {
            Specific(p) => p as i16,
            All => -1,
        }
    }

    /// Returns the raw, C-FFI compatible Note ID component of this PCKN.
    ///
    /// This returns `-1` if the Note ID is set to [`Match::All`], otherwise the specific value is
    /// returned.
    #[inline]
    pub const fn raw_note_id(&self) -> i32 {
        match self.note_id {
            Specific(p) => p as i32,
            All => -1,
        }
    }
}

/// Represents matching either a specific value or all values of a given type.
///
/// This is used in the [`Pckn`] type to support matching multiple kinds of notes at once.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Match<T> {
    /// Matches a specific value.
    Specific(T),
    /// Matches all values.
    All,
}

impl<T> From<T> for Match<T> {
    #[inline]
    fn from(value: T) -> Self {
        Specific(value)
    }
}

impl From<u8> for Match<u16> {
    #[inline]
    fn from(value: u8) -> Self {
        Specific(value.into())
    }
}

impl From<u8> for Match<u32> {
    #[inline]
    fn from(value: u8) -> Self {
        Specific(value.into())
    }
}

impl From<u16> for Match<u32> {
    #[inline]
    fn from(value: u16) -> Self {
        Specific(value.into())
    }
}

impl<T: PartialEq> Match<T> {
    /// Returns `true` if the given [`Match`] matches this one, `false` otherwise.
    ///
    /// This will always return true if any of the two is [`Match::All`]. Otherwise, if both values
    /// are specific, they are compared directly (using their [`PartialEq`] implementation).
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::Match;
    ///
    /// assert!(Match::Specific(42).matches(&Match::Specific(42)));
    /// assert!(!Match::Specific(42).matches(&Match::Specific(21)));
    ///
    /// assert!(Match::Specific(42).matches(&Match::All));
    /// assert!(Match::All.matches(&Match::Specific(42)));
    /// assert!(Match::<u16>::All.matches(&Match::All));
    /// ```
    #[inline]
    pub fn matches(&self, other: &Match<T>) -> bool {
        match (self, other) {
            (Specific(x), Specific(y)) => x == y,
            _ => true,
        }
    }
}

impl Match<u16> {
    /// Creates the [`Match`] that corresponds to the given raw C-FFI compatible `i16` type.
    #[inline]
    pub const fn from_raw(raw: i16) -> Self {
        if raw < 0 {
            All
        } else {
            Specific(raw as u16)
        }
    }
}

impl Match<u32> {
    /// Creates the [`Match`] that corresponds to the given raw C-FFI compatible `i32` type.
    #[inline]
    pub const fn from_raw(raw: i32) -> Self {
        if raw < 0 {
            All
        } else {
            Specific(raw as u32)
        }
    }
}