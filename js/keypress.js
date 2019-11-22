var keypress_timers = {};
var keypresses = {};

// gets whichever keys were pressed since the last call of this function
function getKeypresses() {
    let keypresses_buffer = Object.assign({}, keypresses); // shallow copy
    keypresses = {};

    return keypresses_buffer;
}

function initKeypressHandler() {
    // keydown map contains whether or not each key is down
    window.addEventListener('keydown', (e) => {
        // if there's already a timer to fire this keypress, do nothing
        if (!(e.key in keypress_timers)) { 
            // trigger a keypress event immediately
            keypresses[e.key]=true;

            console.log(`${e.key}: first press event`);

            // then trigger another each KEYPRESS_INTERVAL seconds
            keypress_timers[e.key] = setInterval(() => {
                console.log(`${e.key}: next press event`);
                keypresses[e.key]=true;
            }, KEYPRESS_INTERVAL);
        }

        e.preventDefault();
    });

    window.addEventListener('keyup', (e) => {
        // this may be undefined if the key was pressed before the
        // program started
        if (e.key in keypress_timers) {
            console.log(`${e.key}: stop press events`);
            clearInterval(keypress_timers[e.key]);
            delete keypress_timers[e.key];
        }

        e.preventDefault();
    });
}
