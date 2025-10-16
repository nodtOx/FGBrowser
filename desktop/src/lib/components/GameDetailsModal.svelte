<script lang="ts">
    import { selectedGame } from '$lib/stores/games';
    import { closeGameDetails, showGameDetails } from '$lib/stores/navigation';
    import { onDestroy } from 'svelte';
    import GameDetails from './GameDetails.svelte';

    function handleBackdropClick(event: MouseEvent) {
        if (event.target === event.currentTarget) {
            closeGameDetails();
        }
    }

    function handleKeydown(event: KeyboardEvent) {
        // Check if user is typing in an input field
        const target = event.target as HTMLElement;
        const isTyping = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
        
        // Don't close modal with Backspace if user is typing (but allow Escape)
        if (event.key === 'Escape') {
            event.preventDefault();
            closeGameDetails();
        } else if (event.key === 'Backspace' && !isTyping) {
            event.preventDefault();
            closeGameDetails();
        }
    }

    // Add/remove event listener based on modal visibility
    $: {
        if ($showGameDetails) {
            window.addEventListener('keydown', handleKeydown);
        } else {
            window.removeEventListener('keydown', handleKeydown);
        }
    }

    onDestroy(() => {
        window.removeEventListener('keydown', handleKeydown);
    });
</script>

{#if $showGameDetails && $selectedGame}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div 
        class="modal-backdrop" 
        on:click={handleBackdropClick}
    >
        <div class="modal-content">
            <div class="modal-header">
                <h2>Game Details</h2>
                <button class="close-button" on:click={closeGameDetails}>×</button>
            </div>
            <div class="modal-body">
                <GameDetails />
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 20px;
    }

    .modal-content {
        background: var(--color-background);
        border-radius: 8px;
        max-width: 800px;
        width: 100%;
        max-height: 90vh;
        overflow: hidden;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
        border: 1px solid var(--color-border);
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 20px 24px;
        border-bottom: 1px solid var(--color-border);
        background: var(--color-backgroundSecondary);
    }

    .modal-header h2 {
        margin: 0;
        font-size: 20px;
        font-weight: 600;
        color: var(--color-text);
    }

    .close-button {
        background: none;
        border: none;
        font-size: 24px;
        cursor: pointer;
        color: var(--color-textSecondary);
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        transition: all 0.2s ease;
    }

    .close-button:hover {
        background: var(--color-hover);
        color: var(--color-text);
    }

    .modal-body {
        overflow-y: auto;
        max-height: calc(90vh - 80px);
    }
</style>
