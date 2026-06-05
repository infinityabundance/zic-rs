import sys, lzma, struct
def unlzip(path):
    data=open(path,'rb').read()
    out=bytearray(); pos=0
    while pos < len(data):
        if data[pos:pos+4]!=b'LZIP': break
        ver=data[pos+4]; b=data[pos+5]
        base=b & 0x1f
        dict_size=(1<<base)
        frac=(b>>5)&7
        dict_size-=(dict_size//16)*frac
        # member trailer: CRC(4)+data_size(8)+member_size(8) at end of THIS member
        # find member_size by reading from the end is hard for concatenated members;
        # tz bundles are single-member, so decode from pos+6 to end-20.
        # but to be safe, decode the LZMA1 stream with a decompressor that stops at stream end.
        filt=[{"id":lzma.FILTER_LZMA1,"dict_size":dict_size,"lc":3,"lp":0,"pb":2}]
        dec=lzma.LZMADecompressor(format=lzma.FORMAT_RAW, filters=filt)
        # feed everything from pos+6; decoder stops at end marker; leftover is trailer+next member
        chunk=dec.decompress(data[pos+6:])
        out+=chunk
        # advance: unused_data holds bytes after the LZMA end marker (the 20-byte trailer + any next member)
        consumed=len(data)-pos-6-len(dec.unused_data)
        # member = magic(6)+lzma(consumed)+trailer(20)
        pos=pos+6+consumed+20
    return bytes(out)
sys.stdout.buffer.write(unlzip(sys.argv[1]))
