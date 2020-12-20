# Structure of compressed files

I suggest to use `.isvd` for compressed files containing image and `.wsvd` for compressed files containing sound. 

## Header

| Type to read | Name   | Description |
| ------------ | ------ | ----------- |
| `u8`         | `type` | The type of content of the file. 1st LSB: wether or not we use `f32` variables, 2nd LSB: wether or not this is an audio file (thus, an audio header is present) |
| `[u8; 20]?` | `a_h` | If audio file (thus optionnal), the WAV audio header that has to be losslessly preserved |
| `u32`  | `n`      | Number of triplets `sv_i`, `u_i`, `v_t_i` stored in that file |
| `u32`  | `height` | The number of rows of the matrix, that is twice the height of the image |
| `u32`  | `width`  | The number of columns of the matirx, that is twice the width of the image |

## Body

`n` times: triplet `sv_i`, `u_i`, `v_t_i` 

| Type to read | Name   | Description |
| ------------ | ------ | ----------- |
| `f64`        | `sv_i` | The i-th singular value |
| `[f64; height]` | `u_i`   | The i-th left singular vector |
| `[f64; width]`  | `v_t_i` | The i-th right singular vector |