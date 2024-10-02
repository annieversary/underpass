const resizer = document.querySelector("#resizer");
const left: HTMLDivElement = document.querySelector("#left");
const right: HTMLDivElement = document.querySelector("#right");

function resize(e: { x: number }) {
    left.style.width = `${e.x}px`;
    right.style.width = `calc(100vw - ${e.x}px)`;
    window.localStorage.setItem('codeWidth', e.x.toString());
}

const codeWidth = +window.localStorage.getItem('codeWidth') || (window.innerWidth * 0.3);
// so for some reason, when the codeWidth is too small, the map doesnt fill all of the available space
// resizing does force it to take all the space available, so we set it to +1, then immediately set it to the correct value
// this makes it work as expected tho it's *slightly* hacky and funky
resize({ x: codeWidth + 1 });
setTimeout(() => {
    resize({ x: codeWidth });
}, 100);

resizer.addEventListener("mousedown", () => {
    document.addEventListener("mousemove", resize, false);
    document.addEventListener("mouseup", () => {
        document.removeEventListener("mousemove", resize, false);
    }, false);
});
