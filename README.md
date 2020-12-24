# svd-img

A simple program in rust to compress images or WAV files using SVD approximation.

It is also possible to create reduced version of already compressed files.

I guess there are better ways to do as I just write numbers in the compressed files, it could store many zeros so that it's not that efficient (you can set either simple or double precision, double is the default setting):-(

This is providided without any warranty of any kind.

Based on crates `image` and `nalgebra` and `wav`.

## Description
Usage:
```
svd-img [FLAGS] [OPTIONS] <input> <output>
```
If there is no flags setting the mode, it will be deduced from the `<input>` argument: 
- It ends with `.wav` or `.WAV`: set mode to encode with wav input
- Else: set mode to decode with image input

## Examples

Compress an image by 50%:

```
svd-img image.png compressed-image.isvd -p 50
```

Compress an image and keep one pair of vectors (its the maximum compression you can get):
```
svd-img image.png compressed-image.isvd -n 1
```

Compress a WAV file using default compression (25%):
```
svd-img sound.wav compressed-sound.wsvd
svd-img sound.strange compressed-sound.wsvd -W
```

Decode a compressed file containig an image, and a sound:
```
svd-img compressed-image.isvd image.png
svd-img compressed-sound.wsvd sound.wav
```

Reduce a compressed file:
```
svd-img compressed-thing.svd more-compressed-thing.svd -r -p 50
svd-img compressed-thing.svd more-compressed-thing.svd -r -n 3
```

## Flags
| Long name   | Short | Description |
| ----------- | ----- | ----------- |
| `--help`    | `-h`    | Prints the help. |
| `--encode`  | `-e`    | Sets the mode to encode. Clashes with `-d` and `-r`. |
| `--decode`  | `-d`    | Sets the mode to decode. Clashes with `-e` and `-r`. |
| `--reduce`  | `-r`    | Sets the mode to reduce. Clashes with `-e` and `-d`. |
| `--simple-precision` | `-4` | Use simple precision floating point values in the computations. |
| `--double-precision` | `-8` | Use double precision floating point values in the computations. |
| `--version` | `-V`    | Prints version information (quite useless cuz it will remain 0.1). |
| `--wav-input` | `-W`  | Consider the input file as a WAV file, whatever its name. |
| `--no-aggregate` | `-s` | Disable the aggregation of pixels components (r, g, b, a?) into one single `i32` value. |
| `--with-alpha` | `-a` | Adds an alpha channel to the compressed image |

## Options
| Long name   | Short | Description |
| ---------   | ----- | ----------- |
| `--compression-%` | `-p` | Sets the compression ratio, in percentage. Clashes with `-n`. |
| `--num-vectors` | `-n` | Sets the number of vectors to store in the compressed file. Clashes with `-p`. |
| `--epsilon` | `-E`  | Sets the epsilon used for the computation of the SVD. That is, the value used to determine if a value converged to 0. Default is 1e-5. |
| `--n-iter`  | `-i`  | Sets the maximum number of iteration allowed for the computation of the SVD. Default is 0 (what means until convergence) |

## Note
Using `--simple-precition` may lead to some errors if dealing with an alpha channel (`--with-alpha`) with aggregation (by default, use `--no-aggregate` to disable), as there is a 23-bit long fractionnal part, but per pixel we have `4 * 8 = 32` bits of payload. It's problematic, isn't it ? (Remember that the first bit is implied so that storing `3 * 4 = 24` bits should be ok, though I have no idea of the precision that may be lost during the computation of the SVD)

The aggregation of pixels in one value is done to minimize th impact of this on one specific channels. If `x_k` denotes the `k`-th least significant bit of channel `x`, starting from the most significant bit of the result: 
`r_8 g_8 b_8 a_8 r_7 g_7 b_7 a_7 ...`


------------------------------
Ye man at first it only compressed images, but I had the idea to do the same with sound so that I just added that feature. The *img* nows means: I'M Gone. (Btw, when I'm gone, no need to wonder if I ever think of you)