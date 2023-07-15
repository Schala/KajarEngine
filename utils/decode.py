import sys, struct, zlib
from collections import namedtuple

Header = namedtuple('Header', ['signature','size','offset','compressed_size'])
Header.format = '>I<III'

Entry = namedtuple('Entry', ['path_offset','offset','size'])
Entry.format = '<III'

def decode(buf, offset=0):
	seed = 0x19000000 + offset
	out = b''
	for b in buf:
		seed = (seed * 0x41C64E6D + 12345) & 0xFFFFFFFF
		out += (b ^ (seed >> 24)).to_bytes(1)
	return out

if len(sys.argv) > 1:
	with open(sys.argv[1], "r+b") as f:
		buf = f.read()

		with open(sys.argv[1] + ".out", "w+b") as f2:
			buf2 = decode(buf[0:16])
			#f2.write(buf2)
			header = Header(*struct.unpack('<IIII', buf2))
			buf2 = decode(buf[(header.offset):], header.offset)
			dcmp = zlib.decompress(buf2[4:], 31)
			nentries = struct.unpack_from('<I', dcmp, 0)[0]
			dcmp = dcmp[4:]
			entries = []

			for i in range(0, nentries*12, 12):
				entries.append(Entry(*struct.unpack_from('<III', dcmp, i)))

			paths = []
			for e in entries:
				paths.append(struct.unpack_from('s', dcmp, e.path_offset)[0].decode('utf-8'))
			for p in paths:
				print(p)
			#f2.write(dcmp)
