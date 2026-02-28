function stemFromUrl() {
    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
    return m ? decodeURIComponent(m[2]) : null;
}

function syncView(scroll) {
    var stem = stemFromUrl();
    document.getElementById('sidebar').classList.toggle('mobile-hidden', !!stem);
    document.getElementById('note-content').classList.toggle('mobile-visible', !!stem);
    document.querySelectorAll('.note-item.active-note').forEach(function(el) {
        el.classList.remove('active-note');
    });
    if (stem) {
        var el = document.querySelector('.note-item[data-stem="' + CSS.escape(stem) + '"]');
        if (el) {
            el.classList.add('active-note');
            if (scroll) el.scrollIntoView({block: 'nearest'});
        }
    }
    var h2 = document.querySelector('#note-content h2');
    document.title = h2 ? h2.textContent + ' \u{2013} weave' : 'weave';
}

function showNote(e) {
    if (e && e.currentTarget) {
        document.querySelectorAll('.note-item.active-note').forEach(function(el) {
            el.classList.remove('active-note');
        });
        e.currentTarget.classList.add('active-note');
        e.currentTarget.style.position = 'relative';
    }
}

function showList() {
    document.getElementById('sidebar').classList.remove('mobile-hidden');
    document.getElementById('note-content').classList.remove('mobile-visible');
}

var hasAppHistory = location.pathname === '/';

document.addEventListener('htmx:pushedIntoHistory', function() { hasAppHistory = true; });

function goBack() {
    if (hasAppHistory) {
        history.back();
    } else {
        location.href = '/';
    }
}

function toggleFocus() {
    if (window.innerWidth < 768) return;
    var prose = document.querySelector('#note-content .prose');
    if (!document.body.classList.contains('focus-mode')) {
        if (prose) {
            document.body.style.setProperty('--focus-prose-width', prose.offsetWidth + 'px');
        }
        document.body.classList.add('focus-mode');
    } else {
        if (prose) {
            prose.style.margin = '0 auto';
            prose.style.maxWidth = document.body.style.getPropertyValue('--focus-prose-width') || '65ch';
        }
        document.body.classList.remove('focus-mode');
        document.getElementById('sidebar').addEventListener('transitionend', function handler() {
            document.body.style.removeProperty('--focus-prose-width');
            if (prose) {
                prose.style.margin = '';
                prose.style.maxWidth = '';
            }
            document.getElementById('sidebar').removeEventListener('transitionend', handler);
        });
    }
}

document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
        if (document.body.classList.contains('focus-mode')) {
            toggleFocus();
            return;
        }
        if (document.activeElement === document.getElementById('filter-input')) {
            document.activeElement.blur();
            return;
        }
    }
    var t = document.activeElement.tagName;
    if (t === 'INPUT' || t === 'TEXTAREA' || t === 'SELECT') return;
    if (e.key === 'e') {
        var btn = document.querySelector('[aria-label="Edit note"]');
        if (btn) btn.click();
    } else if (e.key === 'f') {
        e.preventDefault();
        toggleFocus();
    } else if (e.key === 's') {
        e.preventDefault();
        document.getElementById('filter-input').focus();
    } else if (e.key === 'j' || e.key === 'k') {
        var items = Array.from(document.querySelectorAll('#search-list .note-item'));
        if (!items.length) return;
        var active = document.querySelector('.note-item.active-note');
        var idx = active ? items.indexOf(active) : -1;
        var next = e.key === 'j' ? idx + 1 : idx - 1;
        if (next < 0 || next >= items.length) return;
        items[next].click();
        items[next].scrollIntoView({block: 'nearest'});
    }
});

document.addEventListener('DOMContentLoaded', function() { syncView(true); });

window.addEventListener('popstate', function() { syncView(true); });

document.addEventListener('htmx:beforeHistorySave', function() {
    // Only sync mobile classes for the snapshot; skip active-note
    // to avoid re-marking the old note (showNote already set it).
    var stem = stemFromUrl();
    document.getElementById('sidebar').classList.toggle('mobile-hidden', !!stem);
    document.getElementById('note-content').classList.toggle('mobile-visible', !!stem);
});

document.addEventListener('htmx:historyRestore', function() {
    syncView(true);
});

document.addEventListener('htmx:afterSettle', function(e) {
    if (e.detail.target.id === 'note-content') {
        syncView(false);
    }
});
