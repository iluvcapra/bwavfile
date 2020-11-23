/*! 
# bwavfile
 
Rust Wave File Reader/Writer with Broadcast-WAV, MBWF and RF64 Support
 
__(Note: This crate is still in an alpha or pre-alpha stage of development. Reading of
files works however the interfaces may change significantly. Stay up-to-date on the
status of this project at [Github][github].)__

## Objectives and Roadmap

This package aims to support read and writing any kind of WAV file you are likely 
to encounter in a professional audio, motion picture production, broadcast, or music 
production.

Apps we test against:
- Avid Pro Tools
- FFMpeg
- Audacity

Wave features we want to support with maximum reliability and ease of use:

- Large file size, RF64 support
- Multichannel audio formats 
- Embedded metadata

In addition to reading the audio, we want to support all of the different 
metadata planes you are liable to need to use.

- Broadcast-WAV metadata (including the SMPTE UMID and EBU v2 extensions)
- iXML Production recorder metadata
- ADM XML (with associated `chna` mappings)
- Dolby metadata block

Things that are _not_ necessarily in the scope of this package:

- Broad codec support. There are a little more than one-hundred 
  [registered wave codecs][rfc3261], but because this library is targeting
  professional formats being created today, we only plan on supporting
  two of them: tag 0x0001 (Integer Linear PCM) and tag 0x0003 (IEEE Float 
  Linear PCM).
- Music library metadata. There are several packages that can read ID3
  metadata and it's not particuarly common in wave files in any case. INFO
  metadata is more common though in professional applications it tends not
  to be used by many applications.


## Resources

### Implementation of Broadcast Wave Files
- [EBU Tech 3285][ebu3285] (May 2011), "Specification of the Broadcast Wave Format (BWF)"

### Implementation of 64-bit Wave Files
- [ITU-R 2088][itu2088] (October 2019), "Long-form file format for the international exchange of audio programme materials with metadata"
  - Presently in force, adopted by the EBU in [EBU Tech 3306v2][ebu3306v2] (June 2018).
- [EBU Tech 3306v1][ebu3306v1] (July 2009), "MBWF / RF64: An extended File Format for Audio"
  - No longer in force, however long-established. 


### Implementation of Wave format `fmt` chunk
- [MSDN WAVEFORMATEX](https://docs.microsoft.com/en-us/windows/win32/api/mmeapi/ns-mmeapi-waveformatex)
- [MSDN WAVEFORMATEXTENSIBLE](https://docs.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatextensible)


### Other resources
- [RFC 3261][rfc3261] (June 1998) "WAVE and AVI Codec Registries" 
- [Peter Kabal, McGill University](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html)
 - [Multimedia Programming Interface and Data Specifications 1.0](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/Docs/riffmci.pdf) 
    IBM Corporation and Microsoft Corporation, (August 1991)


[ebu3285]: https://tech.ebu.ch/docs/tech/tech3285.pdf
[ebu3306v1]: https://tech.ebu.ch/docs/tech/tech3306v1_1.pdf
[ebu3306v2]: https://tech.ebu.ch/docs/tech/tech3306.pdf
[itu2088]: https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.2088-1-201910-I!!PDF-E.pdf
[rfc3261]: https://tools.ietf.org/html/rfc2361
[github]: https://github.com/iluvcapra/bwavfile
*/

// #![feature(external_doc)]

// #[doc(include="../README.md")]
// #[cfg(doctest)]
// pub struct ReadmeDoctests;

extern crate encoding;
extern crate byteorder;

mod parser;
mod fourcc;
mod errors;

mod validation;

mod raw_chunk_reader;
mod audio_frame_reader;
mod chunks;

mod wavereader;
mod wavewriter;

pub use wavereader::{WaveReader};
pub use chunks::{WaveFmt,Bext};
pub use errors::Error;
pub use audio_frame_reader::AudioFrameReader;