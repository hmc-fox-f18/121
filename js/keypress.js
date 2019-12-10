// The keys which -- when depressed -- should trigger repeated keypress events.
// All other events will trigger a keypress event just when they are pressed
// for the first time.
const REPEATED_PRESS_PERIOD = {'ArrowLeft': 100, 'ArrowRight': 100, 'ArrowUp': 200, 'z': 200};

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

            // trigger another keypress event if this key is setup for
            // repeated press event triggering
            if (e.key in REPEATED_PRESS_PERIOD) {
                keypress_timers[e.key] = setInterval(() => {
                    console.log(`${e.key}: next press event`);
                    keypresses[e.key]=true;
                }, REPEATED_PRESS_PERIOD[e.key]);
            }
            else
            {
                // puts the key in keypress_timers so that no future events will be triggered
                keypress_timers[e.key] = null;
            }
        }
    });

    window.addEventListener('keyup', (e) => {
        // this may be undefined if the key was pressed before the
        // program started
        if (e.key in keypress_timers) {
            if (keypress_timers[e.key] != null) {
                console.log(`${e.key}: stop press events`);
                clearInterval(keypress_timers[e.key]);
            }
            delete keypress_timers[e.key];
        }
    });
}
