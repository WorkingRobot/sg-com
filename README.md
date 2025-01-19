# sg-com

An open-source tool to interface with [SG Com](https://www.speech-graphics.com/sg-com-runtime-audio-to-face-animation-software).

## Usage
From an old Fortnite install where SG Com was still used:
- Extract `FortniteGame/Plugins/FacialAnimSystem/Content/Data/algorithms_SGCom2.k` and `FortniteGame/Plugins/FacialAnimSystem/Content/Data/Jonesy.k` from `pakchunk0-WindowsClient.pak` into `deps/`.
- Extract `FortniteGame\Binaries\ThirdParty\SpeechGraphics\Win64\SG_Com.dll` into `deps/`.
- Run with `cargo run`.

## License

This source code (including the ad-hoc `deps/SG_Com.h`) is under the MIT license. Any assets not provided in this repository (like `SG_Com.dll` and all `.k` files) are IP of [Speech Graphics](https://www.speech-graphics.com), so distributing them is at your own discretion. See [LICENSE](LICENSE) for more information.