#!/bin/zsh

mkdir -p media
cd media

# create a silent bext wave file with fixture metadata and a time refernce starting at
# one minute
#
# Keywords for bext metadata are here... 
# https://github.com/FFmpeg/FFmpeg/blob/17a0dfebf55f67653c29a607545a799f12bc0c01/libavformat/wavenc.c#L110
#
ffmpeg -y -f lavfi -i "aevalsrc=0|0:c=stereo" -to 0.1 -ar 48000 -c:a pcm_s24le -write_bext 1 \
    -metadata "description=FFMPEG-generated stereo WAV file with bext metadata" \
    -metadata "originator=ffmpeg" \
    -metadata "originator_reference=STEREO_WAVE_TEST" \
    -metadata "time_reference=2880000" \
    -metadata "origination_date=2020-11-18" \
    -metadata "origination_time=12:00:00" \
    -metadata "umid=0xFF00FF00FF00FF00FF00FF00FF00FF00" \
    -metadata "coding_history=A:PCM,48K" ff_bwav_stereo.wav

ffmpeg -y -f lavfi -i "aevalsrc=0|0|0|0|0|0:c=5.1" -to 0.1 -ar 48000 -c:a pcm_s24le -write_bext 1 \
    -metadata "description=FFMPEG-generated 5.1 WAV file with bext metadata" \
    -metadata "originator=ffmpeg" \
    -metadata "originator_reference=5_1_WAVE_TEST" \
    -metadata "time_reference=0" \
    -metadata "origination_date=2020-11-18" \
    -metadata "origination_time=13:00:00" \
    -metadata "umid=0xFF00FF00FF00FF00FF00FF00FF00FF01" \
    -metadata "coding_history=A:PCM,48K" ff_bwav_51.wav

ffmpeg -y -f lavfi -i "aevalsrc=0" -to 1 -ar 44100 ff_silence.wav

ffmpeg -y -f lavfi -i "aevalsrc=0" -to 1 -ar 44100 -fflags bitexact ff_minimal.wav

# ffmpeg -y -f lavfi -i "aevalsrc=0|0|0|0|0|0:c=5.1" -to 0:45:00 -ar 96000 -c:a pcm_s24le -rf64 1 \
#     -write_bext 1 \
#     -metadata "description=rf64 test file" ff_longfile.wav 

ffmpeg -y -f lavfi -i "anoisesrc=r=48000:a=0.5:c=pink:s=41879" -to 0.1 -ar 48000 -c:a pcm_f32le \
    -write_bext 1 \
    -metadata "description=float test file" ff_float.wav 

touch error.wav

unzip ../arch_pt_media.zip
unzip ../arch_audacity_media.zip
rm -rf __MACOSX