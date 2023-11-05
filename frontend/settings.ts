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

export const settings = {
    hideEmptyNodes: () => document.querySelector<HTMLInputElement>('#settings-hide-empty-nodes').checked,
    tagsShouldHaveQuotes: () => document.querySelector<HTMLInputElement>('#settings-tags-should-have-quotes').checked,
};

function onChangeCheckbox(selector: string, localStorageId: string) {
    const f = document.querySelector<HTMLInputElement>(selector);
    f.checked = window.localStorage.getItem(localStorageId) === 'true';
    f.onchange = function() {
        window.localStorage.setItem(localStorageId, f.checked.toString());
    };
}
onChangeCheckbox('#settings-hide-empty-nodes', 'settings.hideEmptyNodes');
onChangeCheckbox('#settings-tags-should-have-quotes', 'settings.tagsShouldHaveQuotes');
