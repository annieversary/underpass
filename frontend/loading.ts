// loading modal doesnt use openModal cause we want this custom style which looks nicer imo
let loading = false;
let loadingModal: HTMLDivElement | null = document.querySelector("#loading-modal");

/**
 * Enables/disables the loading modal
 */
export function setLoading(value: boolean): void {
    loading = value;
    if (loadingModal) loadingModal.style.display = loading ? 'flex' : 'none';
}

/**
 * Returns true if the application is currently loading, and the loading modal is open
 */
export function isLoading(): boolean {
    return loading;
}
