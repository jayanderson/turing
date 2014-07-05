#!/bin/sh

#WIDTH=1024
#HEIGHT=768
WIDTH=512
HEIGHT=512

# Random noise:
#ffmpeg random input: ffmpeg -y -f rawvideo -s ${WIDTH}x${HEIGHT} -pix_fmt rgb24 -r 24 -i /dev/urandom -an -vcodec mpeg4 random.mp4

# To video:
#./target/turing | ffmpeg -y -f rawvideo -s ${WIDTH}x${HEIGHT} -pix_fmt rgb24 -r 24 -i - -an -vcodec mpeg4 out.mp4

# Play directly:
./target/turing | ffmpeg -y -f rawvideo -s ${WIDTH}x${HEIGHT} -pix_fmt rgb24 -i - -an -f mpeg2video - | vlc -
