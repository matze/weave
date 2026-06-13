// ── helpers ───────────────────────────────────────────────────────────────

function stemFromUrl() {
    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
    return m ? decodeURIComponent(m[2]) : null;
}

function currentArticle() {
    return document.querySelector('#note-content article.note');
}

function currentMode() {
    var a = currentArticle();
    return a ? a.getAttribute('data-mode') : null;
}

// ── view sync ─────────────────────────────────────────────────────────────

function syncView(scroll) {
    var stem = stemFromUrl();
    document.body.toggleAttribute('data-note', !!stem);

    document.querySelectorAll('.note-row.is-active').forEach(function(el) {
        el.classList.remove('is-active');
    });
    if (stem) {
        var row = document.querySelector('.note-row[data-stem="' + CSS.escape(stem) + '"]');
        if (row) {
            row.classList.add('is-active');
            if (scroll) row.scrollIntoView({ block: 'nearest' });
        }
    }

    var article = currentArticle();
    var hasNote = !!article;
    var mode = hasNote ? article.getAttribute('data-mode') : null;

    var segctl = document.getElementById('mode-segctl');
    if (segctl) {
        segctl.querySelectorAll('button').forEach(function(b) {
            b.classList.toggle('is-on', b.getAttribute('data-mode') === mode);
        });
    }
    if (!hasNote && document.documentElement.classList.contains('is-focus')) {
        exitFocus();
    }

    var h1 = document.querySelector('#note-content .note-head h1');
    document.title = h1 ? h1.textContent + ' – weave' : 'weave';
}

function showNote(e) {
    if (e && e.currentTarget) {
        document.querySelectorAll('.note-row.is-active').forEach(function(el) {
            el.classList.remove('is-active');
        });
        e.currentTarget.classList.add('is-active');
        document.body.dataset.note = '1';
    }
}

function showList() { delete document.body.dataset.note; }

// ── heading anchors ─────────────────────────────────────────────────────────

// Put `anchor` into the URL fragment so it can be copied/shared. Use
// replaceState (not location.hash) to avoid a history entry and the popstate
// note reload it would trigger.
function setHash(anchor) {
    try { history.replaceState(null, '', '#' + anchor); } catch (e) {}
}

// Smooth-scroll to a heading and update the URL fragment (TOC clicks).
function gotoHeading(e, anchor) {
    var el = document.getElementById(anchor);
    if (!el) return;
    if (e) e.preventDefault();
    el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    setHash(anchor);
}

// Scroll to the heading named by the current URL fragment, if any.
function scrollToHash() {
    if (location.hash.length <= 1) return;
    var el = document.getElementById(decodeURIComponent(location.hash.slice(1)));
    if (el) el.scrollIntoView({ block: 'start' });
}

function goBack() {
    if (history.length > 1) history.back();
    else location.href = '/';
}

// ── focus mode ────────────────────────────────────────────────────────────

function focusShell() { return document.querySelector('.shell'); }

function enterFocus() {
    if (!currentArticle()) return;
    var shell = focusShell();
    if (!shell) return;
    shell.classList.add('is-focus');
    try { localStorage.setItem('focus', '1'); } catch (e) {}
}

function exitFocus() {
    var shell = focusShell();
    if (!shell) return;
    shell.classList.remove('is-focus');
    try { localStorage.removeItem('focus'); } catch (e) {}
}

function toggleFocus() {
    var shell = focusShell();
    if (!shell) return;
    if (shell.classList.contains('is-focus')) exitFocus();
    else enterFocus();
}

// ── theme ─────────────────────────────────────────────────────────────────

var ICON_MOON = '<svg class="icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>';
var ICON_SUN = '<svg class="icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 1v2"/><path d="M12 21v2"/><path d="M4.22 4.22l1.42 1.42"/><path d="M18.36 18.36l1.42 1.42"/><path d="M1 12h2"/><path d="M21 12h2"/><path d="M4.22 19.78l1.42-1.42"/><path d="M18.36 5.64l1.42-1.42"/><path d="M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10z"/></svg>';

function currentTheme() {
    var stored = null;
    try { stored = localStorage.getItem('theme'); } catch (e) {}
    if (stored === 'dark' || stored === 'light') return stored;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme, persist) {
    if (theme === 'dark' || theme === 'light') {
        document.documentElement.setAttribute('data-theme', theme);
        if (persist) try { localStorage.setItem('theme', theme); } catch (e) {}
    } else {
        document.documentElement.removeAttribute('data-theme');
        if (persist) try { localStorage.removeItem('theme'); } catch (e) {}
    }
    var btn = document.getElementById('theme-toggle');
    if (btn) btn.innerHTML = currentTheme() === 'dark' ? ICON_SUN : ICON_MOON;
}

function toggleTheme() {
    applyTheme(currentTheme() === 'dark' ? 'light' : 'dark', true);
}

// ── clip drawer ───────────────────────────────────────────────────────────

function openClip() {
    var d = document.getElementById('clip-drawer');
    if (!d) return;
    d.classList.add('is-open');
    var i = document.getElementById('clip-input');
    if (i) i.focus();
}

function closeClip() {
    var d = document.getElementById('clip-drawer');
    if (!d) return;
    d.classList.remove('is-open');
    var i = document.getElementById('clip-input');
    if (i) { i.value = ''; i.blur(); }
}

function clipOpen() {
    var d = document.getElementById('clip-drawer');
    return !!(d && d.classList.contains('is-open'));
}

// ── search ────────────────────────────────────────────────────────────────

function focusSearch() {
    var i = document.getElementById('filter-input');
    if (i) { i.focus(); i.select(); }
}

function clearSearch() {
    var i = document.getElementById('filter-input');
    if (!i) return;
    i.value = '';
    htmx.ajax('POST', '/f/search', { target: '#search-list', values: { query: '' } });
    i.blur();
}

// ── mode toggle (Read / Edit) ─────────────────────────────────────────────

function switchMode(mode) {
    var stem = stemFromUrl();
    if (!stem) return;
    if (mode === currentMode()) return;
    if (mode === 'read' && currentMode() === 'edit') {
        // Auto-save: PUT the textarea body; the server returns the read-mode
        // article, which HTMX swaps into #note-content.
        var ta = document.getElementById('editor-textarea');
        htmx.ajax('PUT', '/f/' + encodeURIComponent(stem), {
            target: '#note-content',
            values: { body: ta ? ta.value : '' }
        });
        return;
    }
    var url = mode === 'edit' ? '/f/' + encodeURIComponent(stem) + '/edit'
                              : '/f/' + encodeURIComponent(stem);
    htmx.ajax('GET', url, { target: '#note-content' });
}

// ── raw markdown ──────────────────────────────────────────────────────────

function openRaw() {
    var stem = stemFromUrl();
    if (stem) location.href = '/raw/' + encodeURIComponent(stem);
}

// ── keyboard ──────────────────────────────────────────────────────────────

document.addEventListener('keydown', function(e) {
    var t = document.activeElement;
    var inInput = t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT' || t.isContentEditable);
    var mod = e.ctrlKey || e.metaKey || e.altKey;

    if (e.key === 'Escape') {
        if (clipOpen()) { closeClip(); return; }
        var fi = document.getElementById('filter-input');
        if (fi && fi.value) { clearSearch(); return; }
        if (document.querySelector('.shell.is-focus')) { exitFocus(); return; }
        if (inInput) { t.blur(); return; }
        return;
    }

    if (e.key === 'Tab' && t === document.getElementById('filter-input')) {
        e.preventDefault();
        var first = document.querySelector('#search-list .note-row');
        if (first) {
            first.click();
            first.scrollIntoView({ block: 'nearest' });
        }
        return;
    }

    if ((e.key === 'k' && (e.metaKey || e.ctrlKey)) || (e.key === '/' && !inInput && !mod)) {
        e.preventDefault();
        focusSearch();
        return;
    }

    if (inInput || mod) return;

    switch (e.key) {
        case 'e': e.preventDefault(); switchMode('edit'); return;
        case 'v': e.preventDefault(); switchMode('read'); return;
        case 'f': e.preventDefault(); toggleFocus(); return;
        case 'r': e.preventDefault(); openRaw(); return;
        case 'd': e.preventDefault(); toggleTheme(); return;
        case 'c': {
            var clipBtn = document.getElementById('clip-toggle');
            if (clipBtn) { e.preventDefault(); openClip(); }
            return;
        }
        case 'n': {
            var newBtn = document.getElementById('new-note');
            if (newBtn) { e.preventDefault(); newBtn.click(); }
            return;
        }
        case 'j':
        case 'k': {
            var items = Array.from(document.querySelectorAll('#search-list .note-row'));
            if (!items.length) return;
            var active = document.querySelector('.note-row.is-active');
            var idx = active ? items.indexOf(active) : -1;
            var next = e.key === 'j' ? idx + 1 : idx - 1;
            if (next < 0 || next >= items.length) return;
            items[next].click();
            items[next].scrollIntoView({ block: 'nearest' });
            return;
        }
    }
});

// ── click delegation ──────────────────────────────────────────────────────

document.addEventListener('click', function(e) {
    var heading = e.target.closest('.md h1, .md h2, .md h3, .md h4, .md h5, .md h6');
    if (heading && heading.id && !e.target.closest('a')) { gotoHeading(e, heading.id); return; }

    if (e.target.closest('#clip-toggle')) { openClip(); }
    else if (e.target.closest('#clip-cancel')) { closeClip(); }
    else if (e.target.closest('#theme-toggle')) { toggleTheme(); }
    else if (e.target.closest('#mode-segctl')) {
        var m = currentMode();
        if (m) switchMode(m === 'edit' ? 'read' : 'edit');
    }
});

// ── init ──────────────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', function() {
    applyTheme(currentTheme(), false);
    var persisted = null;
    try { persisted = localStorage.getItem('focus'); } catch (e) {}
    if (persisted) enterFocus();
    syncView(true);
});

// Follow OS preference live, unless user has picked manually
try {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function() {
        var stored = localStorage.getItem('theme');
        if (stored !== 'dark' && stored !== 'light') applyTheme(null, false);
    });
} catch (e) {}

window.addEventListener('popstate', function() {
    var stem = stemFromUrl();
    if (stem) htmx.ajax('GET', '/f/' + encodeURIComponent(stem), { target: '#note-content' });
    syncView(true);
});

document.addEventListener('htmx:historyRestore', function() {
    syncView(true);
});

document.addEventListener('htmx:afterRequest', function(e) {
    if (e.detail.elt && e.detail.elt.id === 'clip-input') closeClip();
});

document.addEventListener('htmx:afterSettle', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        syncView(false);
        scrollToHash();
    }
});

function showNoteError(message) {
    var nc = document.getElementById('note-content');
    if (!nc) return;
    nc.innerHTML =
        '<article class="note"><div class="note-empty">' + message + '</div></article>';
}

document.addEventListener('htmx:sendError', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        showNoteError('note could not be loaded');
    }
});

document.addEventListener('htmx:responseError', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        showNoteError('note could not be loaded');
    }
});

// ── SSE live reload ───────────────────────────────────────────────────────

(function() {
    var source = new EventSource('/events');
    source.addEventListener('notes-updated', function(e) {
        htmx.trigger(document.body, 'notes-updated');
        try {
            var data = JSON.parse(e.data);
            var current = stemFromUrl();
            if (current && current === data.stem) {
                if (data.removed) showNoteError('note was removed');
                else htmx.ajax('GET', '/f/' + encodeURIComponent(data.stem), { target: '#note-content' });
            }
        } catch (err) {}
    });
})();
