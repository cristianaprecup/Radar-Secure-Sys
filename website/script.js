const object = document.getElementById('object');
setInterval(() => {
    fetch('http://192.168.137.134:1234')
        .then(response => response.json())
        .then(data => {
            console.log(data);
            console.log(object.innerText);
            object.innerHTML = `Object found at: ${data.distance} cm`;
            console.log(object.innerText);
            let status_red = document.getElementById('status-red');
            let status_green = document.getElementById('status-green');
            let status_yellow = document.getElementById('status-yellow');
            let r = document.querySelector(':root');
            if (data.distance < 10) {
                status_red.style.display = 'block';
                status_green.style.display = 'none';
                status_yellow.style.display = 'none';
                r.style.setProperty('--bg-circle-color', 'red');
                r.style.setProperty('--background-color', '#ffcccc'); 
            }
            else if (data.distance <= 30) {
                status_red.style.display = 'none';
                status_green.style.display = 'none';
                status_yellow.style.display = 'block';
                r.style.setProperty('--bg-circle-color', 'yellow');
                r.style.setProperty('--background-color', '#ffffcc'); 
            }
            else {
                status_red.style.display = 'none';
                status_green.style.display = 'block';
                status_yellow.style.display = 'none';
                r.style.setProperty('--bg-circle-color', 'green');
                r.style.setProperty('--background-color', '#ccffcc'); 
            }
        });
}, 1000);
