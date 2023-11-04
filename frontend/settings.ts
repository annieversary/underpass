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

// TODO we probably want a way to abstract this when we add more settings keys
const settingsHideEmptyNodes = document.querySelector<HTMLInputElement>('#settings-hide-empty-nodes');
settingsHideEmptyNodes.checked = window.localStorage.getItem('settings.hideEmptyNodes') === 'true';
settingsHideEmptyNodes.onchange = function() {
    window.localStorage.setItem('settings.hideEmptyNodes', settingsHideEmptyNodes.checked.toString());
};
const settingsTagsShouldHaveQuotes = document.querySelector<HTMLInputElement>('#settings-tags-should-have-quotes');
settingsTagsShouldHaveQuotes.checked = window.localStorage.getItem('settings.tagsShouldHaveQuotes') === 'true';
settingsTagsShouldHaveQuotes.onchange = function() {
    window.localStorage.setItem('settings.tagsShouldHaveQuotes', settingsTagsShouldHaveQuotes.checked.toString());
};
