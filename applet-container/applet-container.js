// Applet container element
// Currently only takes a 'src' attribute
class AppletContainer extends HTMLElement {
    //
    // language=CSS
    static style_template = `
        #applet-container {width: 100%;height: 100%;background-color: aqua;display: flex;flex-direction: column;align-items: center;}
        #applet-logo {width: 25%;height: 25%;}
        #applet-start {margin: 1em;}
    `;

    // Shadowroot for this applet, direct child of the <applet-container> element
    #appletShadow;

    // 'container' div for initial content, replaced by applet content once applet is loaded
    #container;
    constructor() {
        super();
    }

    static get observedAttributes() {
        return ['src']  // We don't currently listen to update events
    }

    connectedCallback() {
        if (this.#appletShadow != null) return;   // Don't re-create shadowroot if already initialised before
        this.#appletShadow = this.attachShadow({mode: 'open'});

        this.#container = document.createElement('div');
        this.#container.setAttribute('id', 'applet-container')
        this.shadowRoot.appendChild(this.#container);

        const styleElement = document.createElement('style');
        styleElement.innerText = AppletContainer.style_template;
        this.shadowRoot.appendChild(styleElement)

        const src = this.getAttribute('src');
        if (src === null) {
            this.displayError('No applet specified')
        } else if (typeof this.getAttribute('autoload') === 'string') {
            this.loadApplet(src);
        } else {
            const logo = document.createElement('img');
            logo.setAttribute('id', 'applet-logo');
            logo.setAttribute('src', './logo-placeholder.png');
            logo.setAttribute('alt', 'applet');
            this.#container.appendChild(logo);

            const title = document.createElement('span');
            title.setAttribute('id', 'applet-title');
            title.innerText = "Title"
            this.#container.appendChild(title);

            const size = document.createElement('span');
            size.setAttribute('id', 'applet-size');
            size.innerText = "(unknown size)"
            this.#container.appendChild(size);

            fetch(src, {method: 'HEAD'})
                .then((response) => {
                    const length = response.headers.get('Content-Length')
                    if (length !== null) {
                        let unit = "B";
                        let bytes = Number(length)
                        if (bytes > 1000) {
                            bytes /= 1000;
                            unit = "kB";
                            if (bytes > 1000) {
                                bytes /= 1000;
                                unit = "MB";
                            }
                        }
                        size.innerText = ("(Applet size: " + (Math.round(bytes * 100) / 100) + unit + ")")
                    }
                })

            const loadButton = document.createElement('button');
            loadButton.setAttribute('id', 'applet-start');
            loadButton.innerText = "Load applet";
            this.#container.appendChild(loadButton);

            loadButton.onclick = () => this.loadApplet(src);
        }
    }

    displayError(error) {
        if (typeof error !== 'string') throw new Error("invalid error value type")
        if (this.#container != null) {
            const logo = document.createElement('img');
            logo.setAttribute('id', 'applet-logo');
            logo.setAttribute('src', './logo-placeholder.png');
            logo.setAttribute('alt', 'applet');

            const title = document.createElement('span');
            title.setAttribute('id', 'applet-title');
            title.textContent = "Error: " + error;

            this.#container.replaceChildren(logo, title);
        } else {
            throw new Error("could not display error; displayError called after applet render")
        }
    }

    loadApplet(src) {
        import(src)
            .then(async (module) => {
                await module.default();
                this.#container = null;
                this.#appletShadow.replaceChildren();
                module.__applet_entrypoint(this.#appletShadow);
            })
            .catch((error) => {
                // TODO: Handle error and provide a more useful user-facing error through `displayError`
                console.log(error)
            })
    }
}

window.customElements.define('applet-container', AppletContainer);