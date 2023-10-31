/// throwaway modals
export function openModal(content: string) {
    const modal = document.createElement('div');
    modal.classList.add('modal');
    modal.innerHTML = `
    <div class="modal-content">
        <span class="close">&times;</span>
        <div class="modal-inner">
            ${content}
        </div>
    </div>`;
    document.body.appendChild(modal);
    modal.querySelector<HTMLSpanElement>('span.close').onclick = function() {
        modal.remove();
    };
    window.onclick = function(event: MouseEvent) {
        if (event.target == modal) {
            modal.remove();
        }
    };

    // TODO pressing esc should close the modal
}
