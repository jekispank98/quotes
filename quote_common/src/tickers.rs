//! Ticker symbols and helpers shared between client and server.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use strum_macros::{Display, EnumString};

use crate::error::ParserError;

/// Trait providing file parsing for tickers.
pub trait TickerParser {
    /// Parses tickers from a buffered reader.
    ///
    /// Each non-empty line is parsed as a single `Ticker` value using `FromStr`.
    /// Returns an error if any line cannot be parsed.
    fn parse_from_file<R: BufRead>(reader: R) -> Result<Vec<Ticker>, ParserError>;
}

impl TickerParser for Ticker {
    fn parse_from_file<R: BufRead>(reader: R) -> Result<Vec<Self>, ParserError> {
        let mut tickers = Vec::new();

        for line_result in reader.lines() {
            let line = line_result.map_err(ParserError::Io)?;
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            match trimmed_line.parse::<Self>() {
                Ok(ticker) => tickers.push(ticker),
                Err(e) => return Err(ParserError::ParseTickersFile(e.to_string())),
            }
        }
        Ok(tickers)
    }
}

/// Set of supported ticker symbols.
#[allow(missing_docs)]
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    ValueEnum,
    Display,
    EnumString,
    Hash,
    Eq,
    PartialEq,
)]
#[clap(rename_all = "lower")]
#[strum(ascii_case_insensitive)]
pub enum Ticker {
    AAPL,
    MSFT,
    GOOGL,
    AMZN,
    NVDA,
    META,
    TSLA,
    JPM,
    JNJ,
    V,
    PG,
    UNH,
    HD,
    DIS,
    PYPL,
    NFLX,
    ADBE,
    CRM,
    INTC,
    CSCO,
    PFE,
    ABT,
    TMO,
    ABBV,
    LLY,
    PEP,
    COST,
    TXN,
    AVGO,
    ACN,
    QCOM,
    DHR,
    MDT,
    NKE,
    UPS,
    RTX,
    HON,
    ORCL,
    LIN,
    AMGN,
    LOW,
    SBUX,
    SPGI,
    INTU,
    ISRG,
    T,
    BMY,
    DE,
    PLD,
    CI,
    CAT,
    GS,
    UNP,
    AMT,
    AXP,
    MS,
    BLK,
    GE,
    SYK,
    GILD,
    MMM,
    MO,
    LMT,
    FISV,
    ADI,
    BKNG,
    C,
    SO,
    NEE,
    ZTS,
    TGT,
    DUK,
    ICE,
    BDX,
    PNC,
    CMCSA,
    SCHW,
    MDLZ,
    TJX,
    USB,
    CL,
    EMR,
    APD,
    COF,
    FDX,
    AON,
    WM,
    ECL,
    ITW,
    VRTX,
    D,
    NSC,
    PGR,
    ETN,
    FIS,
    PSA,
    KLAC,
    MCD,
    ADP,
    APTV,
    AEP,
    MCO,
    SHW,
    DD,
    ROP,
    SLB,
    HUM,
    BSX,
    NOC,
    EW,
    UNKNOWN,
}
