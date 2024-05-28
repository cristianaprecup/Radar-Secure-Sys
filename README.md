# Radar Secure System


## Instalation

Before installing this, please make sure that you have installed:  
- Rust
- `elf2uf2-rs` tool  

To successfully run this rust application, you have to follow these steps:

1. Clone the repository
`git clone https://github.com/cristianaprecup/Radar-Secure-Sys.git`

2. Change directory to the project
`cd RadarSecureSys`

3. Change WiFi credentials (please see `src/main.rs:212`)
![image](https://github.com/UPB-FILS-MA/project-cristianaprecup/assets/121363102/82f28911-df68-4ef9-8dcd-925e8962590d)

5. Build the project.
`cargo build`

6. Run the web interface.
> A simple way to do this is by installing the [Live server](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer) extension from VSCode and run the index.html. 

5. In order to be able to flash on the Pico, make sure that you keep pressed the `BOOTSEL` button while plugging it into the PC.

6. Run the command to flash on the Pico
`elf2uf2-rs -s -d .\target\thumbv6m-none-eabi\debug\secure_sys`

**(!)** After flashing, in the console you can see the connection status of Pico. Pico's IP address isn't static so you have to update the `website/script.js:3` so that it fetches the correct IP.
![image](https://github.com/UPB-FILS-MA/project-cristianaprecup/assets/121363102/92d04c46-a7f4-4ed1-a296-38e52de5d9c2)
For the example in this image, the IP that should be fetched is `192.168.137.183:1234`.


## Description

This project aims to develop a radar-like security system using a Raspberry Pi Pico W as its core. Designed to monitor and track the presence of objects within a specified perimeter, the system utilizes ultrasonic sensing technology to detect movements or intrusions.

## Hardware

<!-- Fill out this table with all the hardware components that you mght need.

The format is 
```
| [Device](link://to/device) | This is used ... | [price](link://to/store) |

```

-->

| Device | Usage | Price |
|--------|--------|-------|
| [Raspberry Pi Pico W](https://www.optimusdigital.ro/ro/placi-raspberry-pi/12394-raspberry-pi-pico-w.html) | The microcontroller with Wi-Fi for phone notification | [34.5 lei](https://www.optimusdigital.ro/ro/placi-raspberry-pi/12394-raspberry-pi-pico-w.html) |
| [HC-SR04 Ultrasonic Distance Sensor](https://ardushop.ro/ro/electronica/47-modul-senzor-ultrasonic-detector-distanta.html) | For measuring distances to objects | [20 lei](https://ardushop.ro/ro/electronica/47-modul-senzor-ultrasonic-detector-distanta.html) |
| [SG90 Micro Servo Motor](https://www.optimusdigital.ro/en/servomotors/26-sg90-micro-servo-motor.html) | To rotate the ultrasonic sensor for a wider scan area | [14 lei](https://www.optimusdigital.ro/en/servomotors/26-sg90-micro-servo-motor.html) |
| [Passive Buzzer Module](https://www.optimusdigital.ro/en/electronic-components/12598-passive-buzzer-module.html?search_query=Buzzer&results=87) | For audible alerts | [1 lei](https://www.optimusdigital.ro/en/electronic-components/12598-passive-buzzer-module.html?search_query=Buzzer&results=87) |
| [Tactile Push Button Switch](https://ardushop.ro/ro/home/97-buton-mic-push-button-trough-hole.html?search_query=push+button&results=30) | For manual controls | [2 lei for 2](https://ardushop.ro/ro/home/97-buton-mic-push-button-trough-hole.html?search_query=push+button&results=30) |
| [10KΩ Resistors](https://ardushop.ro/ro/electronica/211-rezistenta-14w-1-buc.html#/96-valoare_rezistenta-10k) | For the buttons | [0.5 lei for 2](https://ardushop.ro/ro/electronica/211-rezistenta-14w-1-buc.html#/96-valoare_rezistenta-10k) |
| [220Ω Resistors](https://ardushop.ro/ro/electronica/211-rezistenta-14w-1-buc.html#/83-valoare_rezistenta-220r) | For the RGB LED | [1 lei for 3](https://ardushop.ro/ro/electronica/211-rezistenta-14w-1-buc.html#/83-valoare_rezistenta-220r) |
| [Breadboard](https://www.bitmi.ro/breadboard-830-puncte-mb-102-10500.html?gad_source=1) | For assembling the prototype | [10 lei](https://www.bitmi.ro/breadboard-830-puncte-mb-102-10500.html?gad_source=1) |
| [Female-to-Male Wires](https://ardushop.ro/ro/electronica/23-40-x-dupont-cables-female-male-10cm.html?search_query=fire&results=203) | For connections | [5 lei per pack](https://ardushop.ro/ro/electronica/23-40-x-dupont-cables-female-male-10cm.html?search_query=fire&results=203) |
| [Female-to-Female Wires](https://www.optimusdigital.ro/en/wires-with-connectors/880-fire-colorate-mama-mama-10p-10-cm.html?search_query=wires&results=565) | For connections | [3 lei per pack](https://www.optimusdigital.ro/en/wires-with-connectors/880-fire-colorate-mama-mama-10p-10-cm.html?search_query=wires&results=565) |
| [Male-to-Male Wires](https://www.optimusdigital.ro/en/wires-with-connectors/885-wires-male-male-10p-10cm.html?search_query=wires&results=565) | For connections | [9 lei for 3 packs](https://www.optimusdigital.ro/en/wires-with-connectors/885-wires-male-male-10p-10cm.html?search_query=wires&results=565) |
| [Micro USB Cable](https://www.optimusdigital.ro/en/usb-cables/4576-cablu-albastru-micro-usb.html?search_query=usb+to+micro+usb&results=516) | To power the Raspberry Pi Pico W | [3 lei](https://www.optimusdigital.ro/en/usb-cables/4576-cablu-albastru-micro-usb.html?search_query=usb+to+micro+usb&results=516) |
| [RGB LED](https://ardushop.ro/ro/electronica/271-led-tricolor-cu-catod-comun.html) | For visual feedback | [2 lei](https://ardushop.ro/ro/electronica/271-led-tricolor-cu-catod-comun.html) |

## Links

<!-- Add a few links that got you the idea and that you think you will use for your project -->

1. [Similar Project](https://www.youtube.com/watch?v=kQRYIH2HwfY&ab_channel=HowToMechatronics)

