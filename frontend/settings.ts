import { dispatchToAllEditors } from './codeEditor/index';
import { Transaction } from "@codemirror/state";
import { vim } from "@replit/codemirror-vim"

import { Compartment } from "@codemirror/state";
import './settings.css';

// settings
// done by hand cause it's not a throwaway modal like with openModal
const settingsButton = document.getElementById('settings-button');
const settingsModal = document.getElementById('settings-modal');
settingsButton.onclick = function() {
    settingsModal.style.display = 'flex';

    settingsModal.querySelector<HTMLSpanElement>('span.close').onclick = function() {
        settingsModal.style.display = 'none';
    };
    window.onclick = function(event: MouseEvent) {
        if (event.target == settingsModal) {
            settingsModal.style.display = 'none';
        }
    };
};

const settingsFields = {
    hideEmptyNodes: {
        elementId: '#settings-hide-empty-nodes',
        type: 'checkbox',
    },
    tagsShouldHaveQuotes: {
        elementId: '#settings-tags-should-have-quotes',
        type: 'checkbox',
    },
    vim: {
        elementId: '#settings-vim',
        type: 'checkbox',
        onChange: (v) => dispatchToAllEditors({ effects: vimCompartment.reconfigure(v ? vim() : []) }),
    },
};

type SettingsOption = keyof typeof settingsFields;
export let settings: { [key in SettingsOption]: () => boolean } = {} as any;
for (const key of Object.keys(settingsFields)) {
    const field = settingsFields[key];
    const f = document.querySelector<HTMLInputElement>(field.elementId);

    switch (field.type) {
        case 'checkbox':
            settings[key] = () => document.querySelector<HTMLInputElement>(field.elementId).checked;

            f.checked = window.localStorage.getItem(`settings.${key}`) === 'true';
            f.onchange = function() {
                window.localStorage.setItem(`settings.${key}`, f.checked.toString());
                if (field.onChange) field.onChange(f.checked);
            };
            break;
    }
}


export const vimCompartment = new Compartment;
