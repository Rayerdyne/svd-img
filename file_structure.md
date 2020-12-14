# Structure of compressed files

## Header

| Type | Name   | Description |
|------|--------|-------------|
| u32  | n      | Number of triplets `sv_i`, `u_i`, `v_t_i` stored in that file |
| u32  | height | The number of rows of the matrix, that is twice the height of the image |
| u32  | width  | The number of columns of the matirx, that is twice the width of the image |

## Body

`n` times: triplet `sv_i`, `u_i`, `v_t_i` 

| Type | Name   | Description |
|------|--------|-------------|
| f64  |  sv_i  | The i-th singular value |
| \[f64; height] | u_i | The i-th left singular vector |
| \[f64; width] | v_t_i | The i-th right singular vector |