#include <xapian.h>

#ifndef _XAPIAN_SHIM_H
#define _XAPIAN_SHIM_H

namespace shim {
  class FfiMatchDecider : public Xapian::MatchDecider {
    public:
      FfiMatchDecider() : Xapian::MatchDecider() {}
      virtual bool operator()(const Xapian::Document &doc) const override { return this->is_match(doc); }
      virtual bool is_match(const Xapian::Document&) const = 0;
  };

  class FfiMatchSpy : public Xapian::MatchSpy {
    public:
      FfiMatchSpy() : Xapian::MatchSpy() {}
      virtual FfiMatchSpy* upcast() { return this; }
      virtual std::string name() const override { return std::string("shim::FfiMatchSpy"); }
      virtual void operator()(const Xapian::Document &doc, double wt) override { return this->observe(doc, wt); }
      virtual void observe(const Xapian::Document&, double) = 0;
  };

  class FfiStopper : public Xapian::Stopper {
    public:
      FfiStopper() : Xapian::Stopper() {}
      virtual bool operator()(const std::string &word) const override { return this->is_stopword(word); }
      virtual bool is_stopword(const std::string&) const = 0;
  };

  inline Xapian::RangeProcessor& date_range_processor_downcast(Xapian::DateRangeProcessor &rp) { return rp; }

  inline Xapian::Document document_copy(const Xapian::Document &doc) { return Xapian::Document(doc); }

  inline void enquire_add_matchspy( Xapian::Enquire &e, FfiMatchSpy *m) { e.add_matchspy(m); }
  inline Xapian::MSet enquire_get_mset(
      const Xapian::Enquire &e, Xapian::doccount first,
      Xapian::doccount maxitems,  Xapian::doccount atleast,
      const Xapian::RSet *rset, const FfiMatchDecider *decider
  ) { return e.get_mset(first, maxitems, atleast, rset, decider); }

  inline Xapian::MSetIterator mset_iterator_copy(const Xapian::MSetIterator &it) { return Xapian::MSetIterator(it); }
  inline void mset_iterator_decrement(Xapian::MSetIterator &it) { it--; }
  inline Xapian::docid mset_iterator_docid(const Xapian::MSetIterator &it) { return *it; }
  inline bool mset_iterator_eq(const Xapian::MSetIterator &a, const Xapian::MSetIterator &b) { return a == b; }
  inline void mset_iterator_increment(Xapian::MSetIterator &it) { it++; }

  inline Xapian::RangeProcessor& number_range_processor_downcast(Xapian::NumberRangeProcessor &rp) { return rp; }

  inline Xapian::PositionIterator position_iterator_copy(const Xapian::PositionIterator &it) { return Xapian::PositionIterator(it); }
  inline bool position_iterator_eq(const Xapian::PositionIterator &a, const Xapian::PositionIterator &b) { return a == b; }
  inline void position_iterator_increment(Xapian::PositionIterator &it) { it++; }
  inline Xapian::termpos position_iterator_position(const Xapian::PositionIterator &it) { return *it; }

  inline void query_parser_set_stopper(Xapian::QueryParser &qp, const FfiStopper *stopper) { qp.set_stopper(stopper); }

  inline std::string stemmer_stem(const Xapian::Stem &stem, const std::string &word) { return stem(word); }

  inline Xapian::TermIterator term_iterator_copy(const Xapian::TermIterator &it) { return Xapian::TermIterator(it); }
  inline bool term_iterator_eq(const Xapian::TermIterator &a, const Xapian::TermIterator &b) { return a == b; }
  inline void term_iterator_increment(Xapian::TermIterator &it) { it++; }
  inline std::string term_iterator_term(const Xapian::TermIterator &it) { return *it; }

  inline const Xapian::Database& writable_database_downcast(const Xapian::WritableDatabase &db) { return db; }
}

#endif
