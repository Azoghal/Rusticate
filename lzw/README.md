# LZW Compression

This is a rust implementation of the LZW compression algorithm.

### Aims

* Fixed or varaible width codes
* Track compression efficiency
* Reset dictionary to adapt if efficiency becomes poor
* Default to normal switch but support early switch encode/decode
* Support MSB and LSB packing order

### Requirements

To facilitate compressing and uncompressing large files, preferable to encode and decode from byte streams. Possibly wrap the bytestream in my own wrapper to deal codes being packed across byte boundaries.

* Read input file
* Encode
    * Build dictionary
    * Track compression efficiency
    * Clear the dictionary
* Decode
    * Rebuild dictionary
        * 2 cases to think of
    * Look out for dictionary clear
* Write to output file
