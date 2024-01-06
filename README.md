# palette-png
대충 만든 wasm용 png 팔레트 조합 라이브러리  
[Online-ModernArt-Maker](https://github.com/hyuckkim/Online-ModernArt-Maker)에서 쓰려고 만들었습니다.

## build
`wasm-pack build --target web --out-dir ../pkg`  
pkg 폴더가 생성됩니다.
## functions

### ten
```rust
ten() -> i32
```
10을 반환합니다.

### quantize
```rust
quantize(
  rawimage: Uint8ClampedArray, 
  image_width: usize, 
  image_height: usize, 
  num_color: u32,
  dithering: f32,
  gamma: f64)
  -> Uint8ClampedArray 
```
이미지 데이터를 quantize 라이브러리를 사용해 픽셀 데이터로 바꿉니다.

rawimage, image_width, image_height는 javascript의 imagedata의 요소들입니다.  
각각 rgba 픽셀들, 길이, 높이입니다.

num_color는 팔레트에서 몇 색을 쓸 지를 나타냅니다.

dithering은 사진의 디터링 품질을 결정합니다. 0에서 1 사이입니다.

gamma는 팔레트 색상 선택 시 민감도를 결정합니다. 0에서 1 사이입니다.

반환된 값은 팔레트를 가지는 png 이미지입니다. 자바스크립트 쪽에서는 그대로 Blob 만들어서 쓰면 됩니다.

### read_palette
```rust
read_palette(
  data: Uint8ClampedArray)
  -> Uint8ClampedArray
```
png 이미지에 포함된 팔레트 개수를 반환합니다.

data는 png 이미지입니다. ` new Uint8ClampedArray(await blob.arrayBuffer)`로 뽑아낼 수 잇습니다.

반환된 값은 팔레트 부분입니다. 구체적으로는 [png 사양](https://www.w3.org/TR/PNG/#5Chunk-layout)의 청크 중 length가 가리키는 값만을 뽑아옵니다.  
따라서 값은 rgb 순서의 3의 배수가 되고 png 파일이 아니거나 팔레트를 사용하지 않으면 빈 배열이 반환됩니다.

### change_palette
```rust
change_palette(
  data: Uint8ClampedArray,
  index: u8,
  r: u8,
  g: u8,
  b: u8)
  -> Uint8ClampedArray
```
특정 번째 팔레트를 해당 r/g/b 값으로 채웁니다.

data는 png 이미지입니다.

index로 몇 번째 팔레트를 변경할 지 선택합니다.  
귀찮아서 팔레트가 몇 개 있는지 확인하는 함수를 안 만들었습니다. 알아서 조심해서 쓰던가 코드 뜯어 고쳐서 쓰세요.  
코드는 270 번째 줄의 length와 index를 277번 줄에서 비교하면 될 것 같습니다.

r,g,b 값은 각각 red, green, blue입니다. [RGB](https://docs.rs/crate/rgb/latest) 라이브러리를 여기에 쓰면 멋질 것 같았는데 안 들어가네요.

반환 값은 팔레트의 특정 부분이 변환된 png 이미지입니다. PLTE 청크가 바뀌었기 때문에 CRC를 다시 진행합니다.  
png 파일이 아니거나 팔레트를 사용하지 않으면 빈 배열이 반환됩니다.
