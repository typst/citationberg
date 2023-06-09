//! CSL constants that describe entries, terms, and variables.

use std::num::IntErrorKind;
use std::{fmt, str::FromStr};

use serde::{de, Deserialize, Deserializer};

/// A CSL variable.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(untagged)]
pub enum Variable {
    /// The set of variables with no other attributes.
    Standard(StandardVariable),
    /// Variables that can be formatted as numbers.
    Number(NumberVariable),
    /// Variables that can be formatted as dates.
    Date(DateVariable),
    /// Variables that can be formatted as names.
    Name(NameVariable),
}

/// The set of variables with no other attributes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StandardVariable {
    /// Abstract of the item (e.g. the abstract of a journal article).
    Abstract,
    /// Short markup, decoration, or annotation to the item (e.g., to indicate
    /// items included in a review); For descriptive text (e.g., in an annotated
    /// bibliography), use note instead.
    Annote,
    /// Archive storing the item.
    Archive,
    /// Collection the item is part of within an archive.
    #[serde(rename = "archive-collection")]
    ArchiveCollection,
    /// Storage location within an archive (e.g. a box and folder number).
    #[serde(rename = "archive-location")]
    ArchiveLocation,
    /// Geographic location of the archive.
    ArchivePlace,
    /// Issuing or judicial authority (e.g. “USPTO” for a patent, “Fairfax
    /// Circuit Court” for a legal case).
    Authority,
    /// Call number (to locate the item in a library).
    CallNumber,
    /// Identifier of the item in the input data file (analogous to BibTeX
    /// entrykey); Use this variable to facilitate conversion between
    /// word-processor and plain-text writing systems; For an identifer intended
    /// as formatted output label for a citation (e.g. “Ferr78”), use
    /// citation-label instead.
    CitationKey,
    /// Label identifying the item in in-text citations of label styles (e.g.
    /// “Ferr78”); May be assigned by the CSL processor based on item metadata;
    /// For the identifier of the item in the input data file, use citation-key
    /// instead.
    CitationLabel,
    /// Title of the collection holding the item (e.g. the series title for a
    /// book; the lecture series title for a presentation).
    CollectionTitle,
    /// Title of the container holding the item (e.g. the book title for a book
    /// chapter, the journal title for a journal article; the album title for a
    /// recording; the session title for multi-part presentation at a
    /// conference).
    ContainerTitle,
    /// Short/abbreviated form of container-title; Deprecated; use
    /// variable="container-title" form="short" instead.
    ContainerTitleShort,
    /// Physical (e.g. size) or temporal (e.g. running time) dimensions of the
    /// item.
    Dimensions,
    /// Minor subdivision of a court with a jurisdiction for a legal item.
    Division,
    /// Digital Object Identifier (e.g. “10.1128/AEM.02591-07”).
    #[serde(rename = "DOI")]
    DOI,
    /// Deprecated legacy variant of event-title.
    Event,
    /// Name of the event related to the item (e.g. the conference name when
    /// citing a conference paper; the meeting where presentation was made).
    EventTitle,
    /// Geographic location of the event related to the item (e.g. “Amsterdam,
    /// The Netherlands”).
    EventPlace,
    /// Type, class, or subtype of the item (e.g. “Doctoral dissertation” for a
    /// PhD thesis; “NIH Publication” for an NIH technical report); Do not use
    /// for topical descriptions or categories (e.g. “adventure” for an
    /// adventure movie).
    Genre,
    /// International Standard Book Number (e.g. “978-3-8474-1017-1”).
    #[serde(rename = "ISBN")]
    ISBN,
    /// International Standard Serial Number.
    #[serde(rename = "ISSN")]
    ISSN,
    /// Geographic scope of relevance (e.g. “US” for a US patent; the court
    /// hearing a legal case).
    Jurisdiction,
    /// Keyword(s) or tag(s) attached to the item.
    Keyword,
    /// The language of the item.
    Language,
    /// The license information applicable to an item (e.g. the license an
    /// article or software is released under; the copyright information for an
    /// item; the classification status of a document).
    License,
    /// Description of the item’s format or medium (e.g. “CD”, “DVD”, “Album”,
    /// etc.).
    Medium,
    /// Descriptive text or notes about an item (e.g. in an annotated
    /// bibliography).
    Note,
    /// Original publisher, for items that have been republished by a different
    /// publisher.
    OriginalPublisher,
    /// Geographic location of the original publisher (e.g. “London, UK”).
    OriginalPublisherPlace,
    /// Title of the original version (e.g. “Война и мир”, the untranslated
    /// Russian title of “War and Peace”).
    OriginalTitle,
    /// Title of the specific part of an item being cited.
    PartTitle,
    /// PubMed Central reference number.
    #[serde(rename = "PMCID")]
    PMCID,
    /// PubMed reference number.
    #[serde(rename = "PMID")]
    PMID,
    /// Publisher.
    Publisher,
    /// Geographic location of the publisher.
    PublisherPlace,
    /// Resources related to the procedural history of a legal case or
    /// legislation; Can also be used to refer to the procedural history of
    /// other items (e.g. “Conference canceled” for a presentation accepted as a
    /// conference that was subsequently canceled; details of a retraction or
    /// correction notice).
    References,
    /// Type of the item being reviewed by the current item (e.g. book, film).
    ReviewedGenre,
    /// Title of the item reviewed by the current item.
    ReviewedTitle,
    /// Scale of e.g. a map or model.
    Scale,
    /// Source from whence the item originates (e.g. a library catalog or
    /// database).
    Source,
    /// Publication status of the item (e.g. “forthcoming”; “in press”; “advance
    /// online publication”; “retracted”).
    Status,
    /// Primary title of the item.
    Title,
    /// Short/abbreviated form of title; Deprecated; use variable="title"
    /// form="short" instead.
    TitleShort,
    /// Uniform Resource Locator (e.g.
    /// “https://aem.asm.org/cgi/content/full/74/9/2766”).
    #[allow(rustdoc::bare_urls)]
    #[serde(rename = "URL")]
    URL,
    /// Title of the volume of the item or container holding the item; Also use
    /// for titles of periodical special issues, special sections, and the like.
    VolumeTitle,
    /// Disambiguating year suffix in author-date styles (e.g. “a” in “Doe,
    /// 1999a”).
    YearSuffix,
}

impl From<StandardVariable> for Variable {
    fn from(value: StandardVariable) -> Self {
        Variable::Standard(value)
    }
}

/// Variables that can be formatted as numbers.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumberVariable {
    /// Chapter number (e.g. chapter number in a book; track number on an
    /// album).
    ChapterNumber,
    /// Index (starting at 1) of the cited reference in the bibliography
    /// (generated by the CSL processor).
    CitationNumber,
    /// Number identifying the collection holding the item (e.g. the series
    /// number for a book).
    CollectionNumber,
    /// (Container) edition holding the item (e.g. “3” when citing a chapter in
    /// the third edition of a book).
    Edition,
    /// Number of a preceding note containing the first reference to the item;
    /// Assigned by the CSL processor; Empty in non-note-based styles or when
    /// the item hasn’t been cited in any preceding notes in a document.
    FirstReferenceNoteNumber,
    /// Issue number of the item or container holding the item (e.g. “5” when
    /// citing a journal article from journal volume 2, issue 5); Use
    /// volume-title for the title of the issue, if any.
    Issue,
    /// A cite-specific pinpointer within the item (e.g. a page number within a
    /// book, or a volume in a multi-volume work); Must be accompanied in the
    /// input data by a label indicating the locator type (see the Locators term
    /// list), which determines which term is rendered by cs:label when the
    /// locator variable is selected.
    Locator,
    /// Number identifying the item (e.g. a report number).
    Number,
    /// Total number of pages of the cited item.
    NumberOfPages,
    /// Total number of volumes, used when citing multi-volume books and such.
    NumberOfVolumes,
    /// Range of pages the item (e.g. a journal article) covers in a container
    /// (e.g. a journal issue).
    Page,
    /// First page of the range of pages the item (e.g. a journal article)
    /// covers in a container (e.g. a journal issue).
    PageFirst,
    /// Number of the specific part of the item being cited (e.g. part 2 of a
    /// journal article); Use part-title for the title of the part, if any.
    PartNumber,
    /// Printing number of the item or container holding the item.
    PrintingNumber,
    /// Section of the item or container holding the item (e.g. “§2.0.1” for a
    /// law; “politics” for a newspaper article).
    Section,
    /// Supplement number of the item or container holding the item (e.g. for
    /// secondary legal items that are regularly updated between editions).
    SupplementNumber,
    /// Version of the item (e.g. “2.0.9” for a software program).
    Version,
    /// Volume number of the item (e.g. “2” when citing volume 2 of a book) or
    /// the container holding the item (e.g. “2” when citing a chapter from
    /// volume 2 of a book); Use volume-title for the title of the volume, if
    /// any.
    Volume,
}

impl From<NumberVariable> for Variable {
    fn from(value: NumberVariable) -> Self {
        Self::Number(value)
    }
}

impl From<NumberVariable> for Term {
    fn from(value: NumberVariable) -> Self {
        Self::NumberVariable(value)
    }
}

/// Variables that can be formatted as dates.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateVariable {
    /// Date the item has been accessed.
    Accessed,
    /// Date the item was initially available (e.g. the online publication date
    /// of a journal article before its formal publication date; the date a
    /// treaty was made available for signing).
    AvailableDate,
    /// Date the event related to an item took place.
    EventFate,
    /// Date the item was issued/published.
    Issued,
    /// Issue date of the original version.
    OriginalFate,
    /// Date the item (e.g. a manuscript) was submitted for publication.
    Submitted,
}

impl From<DateVariable> for Variable {
    fn from(value: DateVariable) -> Self {
        Self::Date(value)
    }
}

/// Variables that can be formatted as names.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameVariable {
    /// Author.
    Author,
    /// The person leading the session containing a presentation (e.g. the
    /// organizer of the container-title of a speech).
    Chair,
    /// Editor of the collection holding the item (e.g. the series editor for a
    /// book).
    CollectionEditor,
    /// Person compiling or selecting material for an item from the works of
    /// various persons or bodies (e.g. for an anthology).
    Compiler,
    /// Composer (e.g. of a musical score).
    Composer,
    /// Author of the container holding the item (e.g. the book author for a
    /// book chapter).
    ContainerAuthor,
    /// A minor contributor to the item; typically cited using “with” before the
    /// name when listed in a bibliography.
    Contributor,
    /// Curator of an exhibit or collection (e.g. in a museum).
    Curator,
    /// Director (e.g. of a film).
    Director,
    /// Editor.
    Editor,
    /// Managing editor (“Directeur de la Publication” in French).
    EditorialDirector,
    /// Combined editor and translator of a work; The citation processory must
    /// be automatically generate if editor and translator variables are
    /// identical; May also be provided directly in item data.
    EditorTranslator,
    /// Executive producer (e.g. of a television series).
    ExecutiveProducer,
    /// Guest (e.g. on a TV show or podcast).
    Guest,
    /// Host (e.g. of a TV show or podcast).
    Host,
    /// Illustrator (e.g. of a children’s book or graphic novel).
    Illustrator,
    /// Interviewer (e.g. of an interview).
    Interviewer,
    /// Narrator (e.g. of an audio book).
    Narrator,
    /// Organizer of an event (e.g. organizer of a workshop or conference).
    Organizer,
    /// The original creator of a work (e.g. the form of the author name listed
    /// on the original version of a book; the historical author of a work; the
    /// original songwriter or performer for a musical piece; the original
    /// developer or programmer for a piece of software; the original author of
    /// an adapted work such as a book adapted into a screenplay).
    OriginalAuthor,
    /// Performer of an item (e.g. an actor appearing in a film; a muscian
    /// performing a piece of music).
    Performer,
    /// Producer (e.g. of a television or radio broadcast).
    Producer,
    /// Recipient (e.g. of a letter).
    Recipient,
    /// Author of the item reviewed by the current item.
    ReviewedAuthor,
    /// Writer of a script or screenplay (e.g. of a film).
    ScriptWriter,
    /// Creator of a series (e.g. of a television series).
    SeriesCreator,
    /// Translator.
    Translator,
}

impl From<NameVariable> for Variable {
    fn from(value: NameVariable) -> Self {
        Self::Name(value)
    }
}

impl From<NameVariable> for Term {
    fn from(value: NameVariable) -> Self {
        Self::NameVariable(value)
    }
}

/// Localizable terms.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
pub enum Term {
    /// Kind of the cited item.
    Kind(Kind),
    /// Variables that can be formatted as names.
    NameVariable(NameVariable),
    /// Variables that can be formatted as numbers.
    NumberVariable(NumberVariable),
    /// A locator.
    Locator(Locator),
    /// Terms that are not defined via types, name or number variables.
    Other(OtherTerm),
}

impl Term {
    /// On which term this falls back if the given term is not available.
    pub const fn term_fallback(self) -> Self {
        match self {
            Self::Other(OtherTerm::LongOrdinal(i)) => Self::Other(OtherTerm::OrdinalN(i)),
            _ => self,
        }
    }

    /// Whether this is an ordinal term.
    pub const fn is_ordinal(self) -> bool {
        matches!(
            self,
            Self::Other(
                OtherTerm::Ordinal | OtherTerm::OrdinalN(_) | OtherTerm::LongOrdinal(_)
            )
        )
    }

    /// Whether this is a gendered term.
    pub const fn is_gendered(self) -> bool {
        if self.is_ordinal() {
            return true;
        };

        matches!(
            self,
            Self::Other(
                OtherTerm::Month01
                    | OtherTerm::Month02
                    | OtherTerm::Month03
                    | OtherTerm::Month04
                    | OtherTerm::Month05
                    | OtherTerm::Month06
                    | OtherTerm::Month07
                    | OtherTerm::Month08
                    | OtherTerm::Month09
                    | OtherTerm::Month10
                    | OtherTerm::Month11
                    | OtherTerm::Month12
            )
        )
    }
}

/// Kind of the cited item.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]

pub enum Kind {
    /// A self-contained work made widely available but not published in a
    /// journal or other publication; Use for preprints, working papers, and
    /// similar works posted on a platform where some level of persistence or
    /// stewardship is expected (e.g. arXiv or other preprint repositories,
    /// working paper series); For unpublished works not made widely available
    /// or only hosted on personal websites, use manuscript.
    Article,
    /// An article published in an academic journal.
    ArticleJournal,
    /// An article published in a non-academic magazine.
    ArticleMagazine,
    /// An article published in a newspaper.
    ArticleNewspaper,
    /// A proposed piece of legislation.
    Bill,
    /// A book or similar work; Can be an authored book or an edited collection
    /// of self-contained chapters; Can be a physical book or an ebook; The
    /// format for an ebook may be specified using medium; Can be a
    /// single-volume work, a multivolume work, or one volume of a multivolume
    /// work; If a container-title is present, the item is interpreted as a book
    /// republished in a collection or anthology; Also used for whole conference
    /// proceedings volumes or exhibition catalogs by specifying event and
    /// related variables.
    Book,
    /// A recorded work broadcast over an electronic medium (e.g. a radio
    /// broadcast, a television show, a podcast); The type of broadcast may be
    /// specified using genre; If container-title is present, the item is
    /// interpreted as an episode contained within a larger broadcast series
    /// (e.g. an episode in a television show or an episode of a podcast).
    Broadcast,
    /// A part of a book cited separately from the book as a whole (e.g. a
    /// chapter in an edited book); Also used for introductions, forewords, and
    /// similar supplemental components of a book.
    Chapter,
    /// A classical or ancient work, sometimes cited using a common
    /// abbreviation.
    Classic,
    /// An archival collection in a museum or other institution.
    Collection,
    /// A data set or a similar collection of (mostly) raw data.
    Dataset,
    /// A catch-all category for items not belonging to other types; Use a more
    /// specific type when appropriate.
    Document,
    /// An entry in a database, directory, or catalog; For entries in a
    /// dictionary, use entry-dictionary; For entries in an encyclopedia, use
    /// entry-encyclopedia.
    Entry,
    /// An entry in a dictionary.
    EntryDictionary,
    /// An entry in an encyclopedia or similar reference work.
    EntryEncyclopedia,
    /// An organized event (e.g., an exhibition or conference); Use for direct
    /// citations to the event, rather than to works contained within an event
    /// (e.g. a presentation in a conference, a graphic in an exhibition) or
    /// based on an event (e.g. a paper-conference published in a proceedings,
    /// an exhibition catalog).
    Event,
    /// A illustration or representation of data, typically as part of a journal
    /// article or other larger work; May be in any format (e.g. image, video,
    /// audio recording, 3D model); The format of the item can be specified
    /// using medium.
    Figure,
    /// A still visual work; Can be used for artwork or other works (e.g.
    /// journalistic or historical photographs); Can be used for any still
    /// visual work (e.g. photographs, drawings, paintings, sculptures,
    /// clothing); The format of the item can be specified using medium.
    Graphic,
    /// A hearing by a government committee or transcript thereof.
    Hearing,
    /// An interview of a person; Also used for a recording or transcript of an
    /// interview; author is interpreted as the interviewee.
    Interview,
    /// A legal case.
    #[serde(rename = "legal_case")]
    LegalCase,
    /// A law or resolution enacted by a governing body.
    Legislation,
    /// An unpublished manuscript; Use for both modern unpublished works and
    /// classical manuscripts; For working papers, preprints, and similar works
    /// posted to a repository, use article.
    Manuscript,
    /// A geographic map.
    Map,
    /// A video or visual recording; If a container-title is present, the item
    /// is interpreted as a part contained within a larger compilation of
    /// recordings (e.g. a part of a multipart documentary)).
    #[serde(rename = "motion_picture")]
    MotionPicture,
    /// The printed score for a piece of music; For a live performance of the
    /// music, use performance; For recordings of the music, use song (for audio
    /// recordings) or motionPicture (for video recordings).
    #[serde(rename = "musical_score")]
    MusicalScore,
    /// A fragment, historical document, or other unusually-published or
    /// ephemeral work (e.g. a sales brochure).
    Pamphlet,
    /// A paper formally published in conference proceedings; For papers
    /// presented at a conference, but not published in a proceedings, use
    /// speech.
    PaperConference,
    /// A patent for an invention.
    Patent,
    /// A live performance of an artistic work; For non-artistic presentations,
    /// use speech; For recordings of a performance, use song or motionPicture.
    Performance,
    /// A full issue or run of issues in a periodical publication (e.g. a
    /// special issue of a journal).
    Periodical,
    /// Personal communications between multiple parties; May be unpublished
    /// (e.g. private correspondence between two researchers) or
    /// collected/published (e.g. a letter published in a collection).
    #[serde(rename = "personal_communication")]
    PersonalCommunication,
    /// A post on a online forum, social media platform, or similar platform;
    /// Also used for comments posted to online items.
    Post,
    /// A blog post.
    PostWeblog,
    /// An administrative order from any level of government.
    Regulation,
    /// A technical report, government report, white paper, brief, or similar
    /// work distributed by an institution; Also used for manuals and similar
    /// technical documentation (e.g. a software, instrument, or test manual);
    /// If a container-title is present, the item is interpreted as a chapter
    /// contained within a larger report.
    Report,
    /// A review of an item other than a book (e.g. a film review, posted peer
    /// review of an article); If reviewed-title is absent, title is taken to be
    /// the title of the reviewed item.
    Review,
    /// A review of a book; If reviewed-title is absent, title is taken to be
    /// the title of the reviewed book.
    ReviewBook,
    /// A computer program, app, or other piece of software.
    Software,
    /// An audio recording; Can be used for any audio recording (not only
    /// music); If a container-title is present, the item is interpreted as a
    /// track contained within a larger album or compilation of recordings.
    Song,
    /// A speech or other presentation (e.g. a paper, talk, poster, or symposium
    /// at a conference); Use genre to specify the type of presentation; Use
    /// event to indicate the event where the presentation was made (e.g. the
    /// conference name); Use container-title if the presentation is part of a
    /// larger session (e.g. a paper in a symposium); For papers published in
    /// conference proceedings, use paper-conference; For artistic performances,
    /// use performance.
    Speech,
    /// A technical standard or similar set of rules or norms.
    Standard,
    /// A thesis written to satisfy requirements for a degree; Use genre to
    /// specify the type of thesis.
    Thesis,
    /// A treaty agreement among political authorities.
    Treaty,
    /// A website or page on a website; Intended for sources which are
    /// intrinsically online; use a more specific type when appropriate (e.g.
    /// article-journal, post-weblog, report, entry); If a container-title is
    /// present, the item is interpreted as a page contained within a larger
    /// website.
    Webpage,
}

impl From<Kind> for Term {
    fn from(value: Kind) -> Self {
        Self::Kind(value)
    }
}

/// A locator.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum Locator {
    Appendix,
    ArticleLocator,
    Book,
    Canon,
    Chapter,
    Column,
    Elocation,
    Equation,
    Figure,
    Folio,
    Issue,
    Line,
    Note,
    Opus,
    Page,
    Paragraph,
    Part,
    Rule,
    Section,
    SubVerbo,
    Supplement,
    Table,
    Timestamp,
    Title,
    Verse,
    Volume,
}

impl From<Locator> for Term {
    fn from(value: Locator) -> Self {
        Self::Locator(value)
    }
}

/// Terms that are not defined via types, name or number variables.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum OtherTerm {
    // Months
    Month01,
    Month02,
    Month03,
    Month04,
    Month05,
    Month06,
    Month07,
    Month08,
    Month09,
    Month10,
    Month11,
    Month12,

    // Ordinals
    Ordinal,
    OrdinalN(u8),
    LongOrdinal(u8),

    // Punctuation
    OpenQuote,
    CloseQuote,
    OpenInnerQuote,
    CloseInnerQuote,
    PageRangeDelimiter,
    Colon,
    Comma,
    Semicolon,

    // Seasons
    Season01,
    Season02,
    Season03,
    Season04,

    // Miscellaneous
    Accessed,
    Ad,
    AdvanceOnlinePublication,
    Album,
    And,
    AndOthers,
    Anonymous,
    At,
    AudioRecording,
    AvailableAt,
    Bc,
    Bce,
    By,
    Ce,
    Circa,
    Cited,
    EtAl,
    Film,
    Forthcoming,
    From,
    Henceforth,
    Ibid,
    In,
    InPress,
    Internet,
    Interview,
    Letter,
    LocCit,
    NoDate,
    NoPlace,
    NoPublisher,
    On,
    Online,
    OpCit,
    OriginalWorkPublished,
    PersonalCommunication,
    Podcast,
    PodcastEpisode,
    Preprint,
    PresentedAt,
    RadioBroadcast,
    RadioSeries,
    RadioSeriesEpisode,
    Reference,
    Retrieved,
    ReviewOf,
    Scale,
    SpecialIssue,
    SpecialSection,
    TelevisionBroadcast,
    TelevisionSeries,
    TelevisionSeriesEpisode,
    Video,
    WorkingPaper,
}

impl FromStr for OtherTerm {
    type Err = TermConversionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parse_int = |s: &str| {
            s.parse::<u8>().map_err(|e| {
                if matches!(
                    e.kind(),
                    IntErrorKind::NegOverflow | IntErrorKind::PosOverflow
                ) {
                    TermConversionError::OutOfRange
                } else {
                    TermConversionError::Unknown
                }
            })
        };

        let month_prefix = "month-";
        let season_prefix = "season-";
        let ordinal_prefix = "ordinal-";
        let long_ordinal_prefix = "long-ordinal-";

        if value.starts_with(month_prefix) {
            let month = parse_int(&value[month_prefix.len()..])?;

            return match month {
                1 => Ok(Self::Month01),
                2 => Ok(Self::Month02),
                3 => Ok(Self::Month03),
                4 => Ok(Self::Month04),
                5 => Ok(Self::Month05),
                6 => Ok(Self::Month06),
                7 => Ok(Self::Month07),
                8 => Ok(Self::Month08),
                9 => Ok(Self::Month09),
                10 => Ok(Self::Month10),
                11 => Ok(Self::Month11),
                12 => Ok(Self::Month12),
                _ => Err(TermConversionError::OutOfRange),
            };
        }

        if value.starts_with(season_prefix) {
            let season = parse_int(&value[season_prefix.len()..])?;

            return match season {
                1 => Ok(Self::Season01),
                2 => Ok(Self::Season02),
                3 => Ok(Self::Season03),
                4 => Ok(Self::Season04),
                _ => Err(TermConversionError::OutOfRange),
            };
        }

        if value.starts_with(ordinal_prefix) {
            let ordinal = parse_int(&value[ordinal_prefix.len()..])?;

            if ordinal > 99 {
                return Err(TermConversionError::OutOfRange);
            }

            return Ok(Self::OrdinalN(ordinal));
        }

        if value.starts_with(long_ordinal_prefix) {
            let ordinal = parse_int(&value[long_ordinal_prefix.len()..])?;

            if ordinal > 10 {
                return Err(TermConversionError::OutOfRange);
            }

            return Ok(Self::LongOrdinal(ordinal));
        }

        match value {
            "ordinal" => Ok(Self::Ordinal),
            "open-quote" => Ok(Self::OpenQuote),
            "close-quote" => Ok(Self::CloseQuote),
            "open-inner-quote" => Ok(Self::OpenInnerQuote),
            "close-inner-quote" => Ok(Self::CloseInnerQuote),
            "page-range-delimiter" => Ok(Self::PageRangeDelimiter),
            "colon" => Ok(Self::Colon),
            "comma" => Ok(Self::Comma),
            "semicolon" => Ok(Self::Semicolon),
            "accessed" => Ok(Self::Accessed),
            "ad" => Ok(Self::Ad),
            "advance-online-publication" => Ok(Self::AdvanceOnlinePublication),
            "album" => Ok(Self::Album),
            "and" => Ok(Self::And),
            "and-others" => Ok(Self::AndOthers),
            "anonymous" => Ok(Self::Anonymous),
            "at" => Ok(Self::At),
            "audio-recording" => Ok(Self::AudioRecording),
            "available at" | "available-at" => Ok(Self::AvailableAt),
            "bc" => Ok(Self::Bc),
            "bce" => Ok(Self::Bce),
            "by" => Ok(Self::By),
            "ce" => Ok(Self::Ce),
            "circa" => Ok(Self::Circa),
            "cited" => Ok(Self::Cited),
            "et-al" => Ok(Self::EtAl),
            "film" => Ok(Self::Film),
            "forthcoming" => Ok(Self::Forthcoming),
            "from" => Ok(Self::From),
            "henceforth" => Ok(Self::Henceforth),
            "ibid" => Ok(Self::Ibid),
            "in" => Ok(Self::In),
            "in press" | "in-press" => Ok(Self::InPress),
            "internet" => Ok(Self::Internet),
            "interview" => Ok(Self::Interview),
            "letter" => Ok(Self::Letter),
            "loc-cit" => Ok(Self::LocCit),
            "no date" | "no-date" => Ok(Self::NoDate),
            "no-place" => Ok(Self::NoPlace),
            "no-publisher" => Ok(Self::NoPublisher),
            "on" => Ok(Self::On),
            "online" => Ok(Self::Online),
            "op-cit" => Ok(Self::OpCit),
            "original-work-published" => Ok(Self::OriginalWorkPublished),
            "personal-communication" => Ok(Self::PersonalCommunication),
            "podcast" => Ok(Self::Podcast),
            "podcast-episode" => Ok(Self::PodcastEpisode),
            "preprint" => Ok(Self::Preprint),
            "presented at" | "presented-at" => Ok(Self::PresentedAt),
            "radio-broadcast" => Ok(Self::RadioBroadcast),
            "radio-series" => Ok(Self::RadioSeries),
            "radio-series-episode" => Ok(Self::RadioSeriesEpisode),
            "reference" => Ok(Self::Reference),
            "retrieved" => Ok(Self::Retrieved),
            "review-of" => Ok(Self::ReviewOf),
            "scale" => Ok(Self::Scale),
            "special-issue" => Ok(Self::SpecialIssue),
            "special-section" => Ok(Self::SpecialSection),
            "television-broadcast" => Ok(Self::TelevisionBroadcast),
            "television-series" => Ok(Self::TelevisionSeries),
            "television-series-episode" => Ok(Self::TelevisionSeriesEpisode),
            "video" => Ok(Self::Video),
            "working-paper" => Ok(Self::WorkingPaper),
            _ => Err(TermConversionError::Unknown),
        }
    }
}

impl<'de> Deserialize<'de> for OtherTerm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

/// An error that can occur when converting a string to a term.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TermConversionError {
    /// The month, season, ordinal, or long ordinal is out of range.
    OutOfRange,
    /// The term is unknown.
    Unknown,
}

impl fmt::Display for TermConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange => write!(f, "value out of range"),
            Self::Unknown => write!(f, "unknown term"),
        }
    }
}

impl From<OtherTerm> for Term {
    fn from(value: OtherTerm) -> Self {
        Self::Other(value)
    }
}
